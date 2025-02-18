use mp3lame_encoder::{Builder, DualPcm, FlushNoGap};

pub fn encode_pcm_to_mp3(left: &[i16], right: &[i16], sample_rate: u32, channels: u8) -> Vec<u8> {
    let mut mp3_encoder = Builder::new().expect("Create LAME builder");

    mp3_encoder
        .set_num_channels(channels)
        .expect("set channels");
    mp3_encoder
        .set_sample_rate(sample_rate)
        .expect("set sample rate");
    mp3_encoder
        .set_brate(mp3lame_encoder::Bitrate::Kbps192)
        .expect("set brate");
    mp3_encoder
        .set_quality(mp3lame_encoder::Quality::Best)
        .expect("set quality");

    let mut mp3_encoder = mp3_encoder.build().expect("To initialize LAME encoder");

    let input = DualPcm { left, right };

    let mut mp3_out_buffer = Vec::new();
    mp3_out_buffer.reserve(mp3lame_encoder::max_required_buffer_size(input.left.len()));

    let encoded_size = mp3_encoder
        .encode(input, mp3_out_buffer.spare_capacity_mut())
        .expect("To encode");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    let encoded_size = mp3_encoder
        .flush::<FlushNoGap>(mp3_out_buffer.spare_capacity_mut())
        .expect("to flush");
    unsafe {
        mp3_out_buffer.set_len(mp3_out_buffer.len().wrapping_add(encoded_size));
    }

    mp3_out_buffer
}
