use std::env;

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


fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args);

    println!("Config: {:?}", config);
}
