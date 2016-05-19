// STD Dependencies -----------------------------------------------------------
use std;
use std::ptr;
use std::cmp;
use std::mem;
use std::fs::File;
use std::io::Write;
use std::io::Error;
use std::ffi::CString;


// Vorbis Dependencies --------------------------------------------------------
use vorbis_sys::{
    vorbis_info,
    vorbis_comment,
    vorbis_dsp_state,
    vorbis_block ,

    vorbis_info_init,
    vorbis_info_clear,
    vorbis_comment_init,
    vorbis_comment_add_tag,
    vorbis_comment_clear,
    vorbis_analysis_init,
    vorbis_analysis,
    vorbis_bitrate_addblock,
    vorbis_bitrate_flushpacket,
    vorbis_analysis_headerout,
    vorbis_analysis_blockout,
    vorbis_analysis_wrote,
    vorbis_analysis_buffer,
    vorbis_dsp_clear,
    vorbis_block_init,
    vorbis_block_clear
};

use vorbisenc_sys::{
    vorbis_encode_init,
    vorbis_encode_init_vbr
};


// Ogg Dependencies -----------------------------------------------------------
use ogg_sys::{
    ogg_stream_state,
    ogg_page,
    ogg_packet,

    ogg_stream_init,
    ogg_stream_clear,
    ogg_stream_packetin,
    ogg_stream_flush,
    ogg_stream_pageout,
    ogg_page_eos
};


// Simple Ogg Vorbis Encoder Implementation -----------------------------------
pub struct OggVorbisEncoder {
    file: File,
    ogg: OggState,
    vorbis: VorbisState
}

impl OggVorbisEncoder {

    pub fn new(filename: &str) -> Result<OggVorbisEncoder, Error> {
        match File::create(filename) {
            Ok(file) => Ok(OggVorbisEncoder {
                file: file,
                ogg: OggState::new(),
                vorbis: VorbisState::new()
            }),
            Err(e) => Err(e)
        }
    }

    pub fn init(&mut self, channels: usize, sample_rate: u32, max_bitrate: Option<u32>, nominal_bitrate: u32, min_bitrate: Option<u32>) {
        self.vorbis.init(
            channels,
            sample_rate as i64,
            max_bitrate.map_or(-1, |b| b as i64),
            nominal_bitrate as i64,
            min_bitrate.map_or(-1, |b| b as i64)
        );
        self.ogg.init(&mut self.vorbis);
        self.ogg.write_header(&mut self.file);
    }

    pub fn init_vbr(&mut self, channels: usize, sample_rate: u32, quality: f32) {
        self.vorbis.init_vbr(channels, sample_rate as i64, quality);
        self.ogg.init(&mut self.vorbis);
        self.ogg.write_header(&mut self.file);
    }

    pub fn write(&mut self, samples: &[i16]) {
        self.vorbis.write_samples(samples);
        self.ogg.write(&mut self.file, &mut self.vorbis);
    }

    pub fn close(&mut self) {
        self.vorbis.close();
        self.ogg.write(&mut self.file, &mut self.vorbis);
    }

}

impl Drop for OggVorbisEncoder {
    fn drop(&mut self) {
        self.ogg.destroy();
        self.vorbis.destroy();
    }
}


// Internal Vorbis Encoding State ---------------------------------------------
#[repr(C)]
struct VorbisState {
    vi: vorbis_info,
    vc: vorbis_comment,
    vd: vorbis_dsp_state,
    vb: vorbis_block,
    channels: usize
}

impl VorbisState {

    fn new() -> VorbisState {
        VorbisState {
            vi: unsafe { mem::zeroed() },
            vc: unsafe { mem::zeroed() },
            vd: unsafe { mem::zeroed() },
            vb: unsafe { mem::zeroed() },
            channels: 0
        }
    }

    fn init(&mut self, channels: usize, sample_rate: i64, max_bitrate: i64, nominal_bitrate: i64, min_bitrate: i64) {
        self.pre_init(channels);
        unsafe {
            vorbis_encode_init(&mut self.vi, channels as i64, sample_rate, max_bitrate, nominal_bitrate, min_bitrate);
        }
        self.post_init();
    }

    fn init_vbr(&mut self, channels: usize, sample_rate: i64, quality: f32) {
        self.pre_init(channels);
        unsafe {
            vorbis_encode_init_vbr(&mut self.vi, channels as i64, sample_rate, quality);
        }
        self.post_init();
    }

    fn write_samples(&mut self, samples: &[i16]) {

        // TODO remove this "fix"...
        // figure out why the vd.vi pointer is not
        // correct after calling "vorbis_analysis_init"
        self.vd.vi = &mut self.vi;

        let len = samples.len() - samples.len() % 8;
        let channel_buffers = unsafe {
            std::slice::from_raw_parts(
                vorbis_analysis_buffer(&mut self.vd, len as i32),
                self.channels
            )
        };

        if self.channels == 1 {

            let ptr: *mut f32 = channel_buffers[0];
            let mono: &mut [f32] = unsafe {
                std::slice::from_raw_parts_mut(ptr, len)
            };

            for i in 0..len {
                mono[i] = samples[i] as f32 / 32768.0;
            }

        } else if self.channels == 2 {

        }

        unsafe {
            vorbis_analysis_wrote(&mut self.vd, (len / self.channels) as i32);
        }

    }

    fn close(&mut self) {
        self.vd.vi = &mut self.vi;
        unsafe {
            vorbis_analysis_wrote(&mut self.vd, 0);
        }
    }

    fn destroy(&mut self) {
        unsafe {
            vorbis_block_clear(&mut self.vb);
            vorbis_dsp_clear(&mut self.vd);
            vorbis_comment_clear(&mut self.vc);
            vorbis_info_clear(&mut self.vi);
        }
    }

    fn pre_init(&mut self, channels: usize) {
        self.channels = channels;
        unsafe {
            vorbis_info_init(&mut self.vi);
            vorbis_comment_init(&mut self.vc);
            vorbis_comment_add_tag(
                &mut self.vc,
                CString::new("ENCODER").unwrap().as_ptr(),
                CString::new("FooBot").unwrap().as_ptr()
            );
        }
    }

    fn post_init(&mut self) {
        unsafe {
            vorbis_analysis_init(&mut self.vd, &mut self.vi);
            vorbis_block_init(&mut self.vd, &mut self.vb);
        }
    }

}


// Internal Ogg Container State -----------------------------------------------
#[repr(C)]
struct OggState {
    os: ogg_stream_state,
    og: ogg_page,
    op: ogg_packet
}

impl OggState {

    fn new() -> OggState {
        OggState {
            os: unsafe { mem::zeroed() },
            og: unsafe { mem::zeroed() },
            op: unsafe { mem::zeroed() }
        }
    }

    fn init(&mut self, vorbis: &mut VorbisState) {

        unsafe {
            // TODO use a random serial number
            ogg_stream_init(&mut self.os, 12345678);
        }

        let mut header: ogg_packet = unsafe { mem::zeroed() };
        let mut header_comm: ogg_packet = unsafe { mem::zeroed() };
        let mut header_code: ogg_packet = unsafe { mem::zeroed() };

        unsafe {
            vorbis_analysis_headerout(
                &mut vorbis.vd,
                &mut vorbis.vc,
                &mut header,
                &mut header_comm,
                &mut header_code
            );
            ogg_stream_packetin(&mut self.os, &mut header);
            ogg_stream_packetin(&mut self.os, &mut header_comm);
            ogg_stream_packetin(&mut self.os, &mut header_code);
        }

    }

    fn write(&mut self, file: &mut File, vorbis: &mut VorbisState) -> bool {

        let null = ptr::null_mut();
        while unsafe { vorbis_analysis_blockout(&mut vorbis.vd, &mut vorbis.vb) } == 1 {

            // Analysis, assume we want to use bitrate management
            unsafe {
                vorbis_analysis(&mut vorbis.vb, null);
                vorbis_bitrate_addblock(&mut vorbis.vb);
            }

            while unsafe { vorbis_bitrate_flushpacket(&mut vorbis.vd, &mut self.op) } != 0 {

                // weld packet into the bitstream
                unsafe {
                    ogg_stream_packetin(&mut self.os, &mut self.op);
                }

                if self.write_page(file) {
                    return true
                }

            }

        }

        false

    }

    fn write_page(&mut self, file: &mut File) -> bool {
        loop {

            let result = unsafe {
                ogg_stream_pageout(&mut self.os, &mut self.og)
            };

            if result == 0 {
                break;

            } else {
                let header: &[u8] = unsafe { std::slice::from_raw_parts(self.og.header, self.og.header_len as usize) };
                let body: &[u8] = unsafe { std::slice::from_raw_parts(self.og.body, self.og.body_len as usize) };
                file.write_all(header).ok();
                file.write_all(body).ok();
            }

            if unsafe { ogg_page_eos(&mut self.og) } != 0 {
                return true;
            }

        }

        false

    }

    fn write_header(&mut self, file: &mut File) {
        loop {

            let result = unsafe {
                ogg_stream_flush(&mut self.os, &mut self.og)
            };

            if result == 0 {
                break;

            } else {
                let header: &[u8] = unsafe { std::slice::from_raw_parts(self.og.header, self.og.header_len as usize) };
                let body: &[u8] = unsafe { std::slice::from_raw_parts(self.og.body, self.og.body_len as usize) };
                file.write_all(header).ok();
                file.write_all(body).ok();
            }
        }
    }

    fn destroy(&mut self) {
        unsafe {
            ogg_stream_clear(&mut self.os);
        }
    }

}

