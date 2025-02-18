mod encoder;

use encoder::encode_pcm_to_mp3;
use rayon::prelude::*;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).expect("file path not provided");

    let (pcm_left, pcm_right, sample_rate) = decode_flac_to_pcm(path);

    println!("Encoding PCM to MP3...");
    let mp3_data = encode_pcm_to_mp3(&pcm_left, &pcm_right, sample_rate, 2);

    println!("Writing MP3 data to file...");
    let mut output_file = File::create("output.mp3").expect("Failed to create output file");
    output_file
        .write_all(&mp3_data)
        .expect("Failed to write MP3 data to file");

    println!("MP3 file created: output.mp3");
}

fn decode_flac_to_pcm(path: &str) -> (Vec<i16>, Vec<i16>, u32) {
    let file = Box::new(File::open(Path::new(path)).expect("Failed to open file"));
    let mss = MediaSourceStream::new(file, Default::default());

    let format_opts: FormatOptions = Default::default();
    let hint = Hint::default();

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &Default::default())
        .expect("Failed to probe format");

    let mut format = probed.format;
    let track = format
        .default_track()
        .expect("No default track found")
        .clone();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .expect("Failed to create decoder");

    let track_id = track.id;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut sample_count = 0;
    let mut pcm_left = Vec::new();
    let mut pcm_right = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(Error::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break
            }
            Err(err) => {
                eprintln!("Error reading packet: {:?}", err);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(audio_buf) => {
                if sample_buf.is_none() {
                    let spec = *audio_buf.spec();
                    let duration = audio_buf.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(audio_buf);
                    sample_count += buf.samples().len();
                    print!("\r=> Decoded {} samples", sample_count);

                    let samples: Vec<i16> = buf
                        .samples()
                        .par_iter()
                        .map(|sample| (*sample * i16::MAX as f32) as i16)
                        .collect();

                    pcm_left.extend(samples.iter().step_by(2).cloned());
                    pcm_right.extend(samples.iter().skip(1).step_by(2).cloned());
                }
            }
            Err(Error::DecodeError(_)) => (),
            Err(err) => {
                eprintln!("Error decoding packet: {:?}", err);
                break;
            }
        }
    }

    println!(
        "\nDecoding completed. Total samples decoded: {}",
        sample_count
    );

    let min_len = pcm_left.len().min(pcm_right.len());
    pcm_left.truncate(min_len);
    pcm_right.truncate(min_len);

    (pcm_left, pcm_right, track.codec_params.sample_rate.unwrap())
}
