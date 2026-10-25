/// Original tiny RIFF fixtures for public audio behavior tests.
pub fn wav(rate: u32, channels: u16, bits: u16, data: &[u8]) -> Vec<u8> {
    let align = channels * (bits / 8);
    let mut out = b"RIFF".to_vec();
    out.extend((36 + data.len() as u32 + (data.len() % 2) as u32).to_le_bytes());
    out.extend(b"WAVEfmt ");
    out.extend(16u32.to_le_bytes());
    out.extend(1u16.to_le_bytes());
    out.extend(channels.to_le_bytes());
    out.extend(rate.to_le_bytes());
    out.extend((rate * u32::from(align)).to_le_bytes());
    out.extend(align.to_le_bytes());
    out.extend(bits.to_le_bytes());
    out.extend(b"data");
    out.extend((data.len() as u32).to_le_bytes());
    out.extend(data);
    if data.len() % 2 == 1 {
        out.push(0)
    }
    out
}
