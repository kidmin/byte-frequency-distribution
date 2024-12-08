type ByteFrequencyTable = [u64; 0x100];

const SCREEN_COLUMNS: usize = 72;

fn usage(progname: &str) {
    eprintln!("usage: {} [input-file | -]", progname);
}

fn read_file_content<T: std::io::Read>(infh: &mut T) -> Result<(Box<ByteFrequencyTable>, u64), std::io::Error> {
    let mut frequency_table = Box::new([0_u64; 0x100]);
    let mut total_bytes = 0_u64;

    let mut read_buffer = vec![0_u8; 1_048_576];
    loop {
        let (bytes_read, should_retry) = match infh.read(&mut read_buffer) {
            Ok(bytes_read) => (bytes_read, false),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (0, true),
            Err(e) => return Err(e),
        };
        if should_retry {
            continue;
        }
        if bytes_read == 0 {
            break;
        }
        total_bytes += bytes_read as u64;
        for b in &read_buffer[0..bytes_read] {
            frequency_table[*b as usize] += 1;
        }
    }

    Ok((frequency_table, total_bytes))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 2 {
        usage(&args[0]);
        std::process::exit(1);
    }

    let (frequency_table, total_bytes) = match args.get(1) {
        Some(filename) if filename != "-" => {
            let mut infh = std::fs::File::open(&filename)?;
            read_file_content(&mut infh)?
        },
        _ => {
            let mut infh = std::io::stdin().lock();
            read_file_content(&mut infh)?
        },
    };

    if total_bytes == 0 {
        eprintln!("(input is empty)");
        std::process::exit(1);
    }

    let frequency_min = *frequency_table.iter().min().unwrap();
    let frequency_max = *frequency_table.iter().max().unwrap();
    let percentage_normalizer = total_bytes as f64 / frequency_max as f64;

    let percentage_min = frequency_min as f64 / total_bytes as f64 * 100.0;
    let percentage_max = frequency_max as f64 / total_bytes as f64 * 100.0;

    let mut outfh = std::io::BufWriter::new(std::io::stdout().lock());
    writeln!(outfh, "(range: {:.2}% - {:.2}%, distribution: {:.2}pt.)", percentage_min, percentage_max, percentage_max - percentage_min)?;
    for b in 0x00_usize..=0xff_usize {
        let frequency = frequency_table[b] as f64 / total_bytes as f64;
        let bar_length = (SCREEN_COLUMNS as f64 * frequency * percentage_normalizer) as usize;
        let bar_str = "*".repeat(bar_length);
        let space_str = " ".repeat(SCREEN_COLUMNS - bar_length);
        writeln!(outfh, "{:02x} |{}{}|{:5.2}%", b, &bar_str, &space_str, frequency * 100.0)?;
    }
    outfh.flush()?;

    Ok(())
}

// vim: set fileencoding=utf-8 nobomb fileformat=unix filetype=rust number expandtab tabstop=8 softtabstop=4 shiftwidth=4 autoindent smartindent :
