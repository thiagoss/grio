use std::env;
extern crate gstreamer;
use gstreamer as gst;
use gst::prelude::*;

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

fn transcode(config : Config) {
    let pipeline = gst::Pipeline::new("transcoder");
    let uridecodebin = gst::ElementFactory::make("uridecodebin", None).unwrap();
    uridecodebin.set_property("uri", &glib::Value::from(&config.input)).unwrap();

    pipeline.add(&uridecodebin).unwrap();

    let pipeline_clone = pipeline.clone();
    uridecodebin.connect_pad_added(move |_, src_pad| {
        let queue = gst::ElementFactory::make("queue", None).unwrap();
        let fakesink = gst::ElementFactory::make("fakesink", None).unwrap();
        let pipeline = &pipeline_clone;

        pipeline.add(&queue).unwrap();
        pipeline.add(&fakesink).unwrap();

        queue.link(&fakesink).unwrap();

        queue.sync_state_with_parent().unwrap();
        fakesink.sync_state_with_parent().unwrap();

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
                println!("Error from {}: {} ({:?})",
                        msg.get_src().get_path_string(), err.get_error(),
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
