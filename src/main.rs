use std::env;
extern crate gstreamer;
use gstreamer as gst;
use gst::prelude::*;

extern crate gstreamer_pbutils;
use gstreamer_pbutils as pbutils;

use pbutils::EncodingContainerProfileExt;

extern crate glib;

#[derive(Debug)]
enum MediaFormat {
    Unknown,
    MP4,
    MKV
}

impl MediaFormat {
    fn from_string(filename: &String) -> MediaFormat {
        let termination_index = filename.rfind(".").unwrap();
        let termination = &filename[termination_index + 1..];
        match termination {
            "mp4" => MediaFormat::MP4,
            "mkv" => MediaFormat::MKV,
            _ => MediaFormat::Unknown
        }
    }
}

#[derive(Debug)]
struct Config {
    input: String,
    output: String,
    output_format: MediaFormat
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        let input = args[1].clone();
        let output = args[2].clone();
        let output_format = MediaFormat::from_string(&output);

        Ok(Config {input, output, output_format})
    }
}

fn create_container_caps(format : &MediaFormat) -> gst::Caps {
    match format {
        &MediaFormat::MP4 => gst::Caps::from_string("video/quicktime, variant=(string)iso").unwrap(),
        &MediaFormat::MKV => gst::Caps::from_string("video/x-matroska").unwrap(),
        &MediaFormat::Unknown => gst::Caps::from_string("").unwrap()
    }
}

fn create_video_caps(format : &MediaFormat) -> gst::Caps {
    gst::Caps::from_string("video/x-h264").unwrap()
}

fn create_audio_caps(format : &MediaFormat) -> gst::Caps {
    gst::Caps::from_string("audio/mpeg, mpegversion=(int)4").unwrap()
}

fn create_encoding_profile(config : &Config) -> pbutils::EncodingProfile {
    let container_caps = create_container_caps(&config.output_format);
    let video_caps = create_video_caps(&config.output_format);
    let audio_caps = create_audio_caps(&config.output_format);

    let encoding_profile = pbutils::EncodingContainerProfile::new("container", None, &container_caps, None);
    let video_profile = pbutils::EncodingVideoProfile::new(&video_caps, None, None, 0);
    let audio_profile = pbutils::EncodingAudioProfile::new(&audio_caps, None, None, 0);
    encoding_profile.add_profile(&video_profile);
    encoding_profile.add_profile(&audio_profile);

    return encoding_profile.upcast();
}

fn transcode(config : Config) {
    let pipeline = gst::Pipeline::new("transcoder");
    let filesink = gst::ElementFactory::make("filesink", None).unwrap();
    let encodebin = gst::ElementFactory::make("encodebin", None).unwrap();
    let uridecodebin = gst::ElementFactory::make("uridecodebin", None).unwrap();
    uridecodebin.set_property("uri", &glib::Value::from(&config.input)).unwrap();
    filesink.set_property("location", &glib::Value::from(&config.output)).unwrap();

    pipeline.add(&uridecodebin).unwrap();
    pipeline.add(&encodebin).unwrap();
    pipeline.add(&filesink).unwrap();

    encodebin.link(&filesink).unwrap();

    let encoding_profile = create_encoding_profile(&config);
    encodebin.set_property("profile", &encoding_profile);

    let pipeline_clone = pipeline.clone();
    let encodebin_clone = encodebin.clone();
    uridecodebin.connect_pad_added(move |_, src_pad| {
        let pipeline = &pipeline_clone;
        let encodebin = &encodebin_clone;

        let (is_audio, is_video) = {
            let caps = src_pad.get_current_caps().unwrap();
            let structure = caps.get_structure(0).unwrap();
            let name = structure.get_name();

            (name.starts_with("audio/"), name.starts_with("video/"))
        };

        let queue = gst::ElementFactory::make("queue", None).unwrap();
        pipeline.add(&queue).unwrap();
        if is_video {
            let enc_sink_pad = encodebin.get_request_pad("video_%u").unwrap();
            let queue_src_pad = queue.get_static_pad("src").unwrap();
            queue_src_pad.link(&enc_sink_pad);
        } else if is_audio {
            let enc_sink_pad = encodebin.get_request_pad("audio_%u").unwrap();
            let queue_src_pad = queue.get_static_pad("src").unwrap();
            queue_src_pad.link(&enc_sink_pad);
        } else {
            return;
        }

        queue.sync_state_with_parent().unwrap();

        let sink_pad = queue.get_static_pad("sink").unwrap();
        assert_eq!(src_pad.link(&sink_pad), gst::PadLinkReturn::Ok);
    });

    pipeline.set_state(gst::State::Playing);
    let bus = pipeline.get_bus().unwrap();

    while let Some(msg) = bus.timed_pop(gst::CLOCK_TIME_NONE) {
        use gst::MessageView;
        match msg.view() {
            MessageView::Eos(..) => break,
            MessageView::Error(err) => {
                println!("Error from {:?}: {} ({:?})",
                        msg.get_src(), err.get_error(),
                        err.get_debug());
                break;
            }
            _ => (),
        }
    }

    pipeline.set_state(gst::State::Null);
}


fn main() {
    gst::init().unwrap();
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap();

    println!("Config: {:?}", config);
    transcode(config);
}
