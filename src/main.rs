use std::env::args;
use std::fs::File;
use std::path::Path;
use std::time::Instant;
use wav::BitDepth;

fn perform_voluming_i<T: Into<f64> + Copy>(data: Vec<T>) -> Vec<i16> {
	data.iter()
		.map(|i| {
			if Into::<f64>::into(*i) > 0.0f64 {
				0x7fff
			} else {
				-0x8000
			}
		})
		.collect()
}

// u8 PCM should be treated other way (if we are going without unsafing (we could transform it into i8)), than a i{16,32} and f32
fn perform_voluming_u8(data: Vec<u8>) -> Vec<i16> {
	data.iter()
		.map(|&i| {
			if i > 128 {
				0x7fff
			} else {
				-0x8000
			}
		})
		.collect()
}

fn main() {
	let args: Vec<String> = args().collect();
	if args.len() != 3 {
		eprintln!("WAR.rs: winner of THE LOUDNESS WAR");
		eprintln!("Usage: {} <INPUT.wav> <OUTPUT.wav>", args[0]);
		std::process::exit(1);
	}
	let (input, output) = (args[1].clone(), args[2].clone());
	let start = Instant::now();
	let mut inp_file = File::open(Path::new(&input)).unwrap();
	let (header, data) = wav::read(&mut inp_file).unwrap();
	let new_header = wav::Header::new(
		wav::header::WAV_FORMAT_PCM,
		header.channel_count,
		header.sampling_rate,
		16,
	);

	let volumed = match data {
		BitDepth::Eight(data) => perform_voluming_u8(data),
		BitDepth::Sixteen(data) => perform_voluming_i(data),
		BitDepth::TwentyFour(data) => perform_voluming_i(data),
		BitDepth::ThirtyTwoFloat(data) => perform_voluming_i(data),
		BitDepth::Empty => {
			eprintln!("No audio data found in input file.");
			std::process::exit(1);
		}
	};

	let mut out_file = File::create(Path::new(&output)).unwrap();
	wav::write(new_header, &BitDepth::Sixteen(volumed.clone()), &mut out_file).unwrap();
	println!("Saved to {} in {} seconds", output, (Instant::now() - start).as_secs_f64());

	let mut ebur =
		ebur128::EbuR128::new(header.channel_count as u32, header.sampling_rate, ebur128::Mode::I)
			.unwrap();
	ebur.add_frames_i16(volumed.as_slice()).unwrap();
	println!("New loudness:\t{} LUFS", ebur.loudness_global().unwrap());
}
