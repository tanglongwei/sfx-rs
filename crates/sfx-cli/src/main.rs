use anyhow::{Context, Result};
use clap::Parser;
use console::style;
use std::fs::{self, File};
use std::io::{Write};
use std::path::{PathBuf};
use tar::{Builder, Header};
use walkdir::WalkDir;
use zstd::stream::write::Encoder;

// Embed the stub binary.
const STUB_BYTES: &[u8] = include_bytes!("../../../target/release/sfx-stub.exe");
const MAGIC: &[u8; 9] = b"SFX_RS_01";
const CONFIG_FILENAME: &str = "__sfx_config";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file or directory to compress
    #[arg(short, long)]
    input: PathBuf,

    /// Output executable name (e.g., installer.exe)
    #[arg(short, long, default_value = "output.exe")]
    output: PathBuf,

    /// Compression level (1-21)
    #[arg(short, long, default_value_t = 3)]
    level: i32,

    /// Command to execute after extraction (optional)
    #[arg(short = 'e', long)]
    exec: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("{} Creating SFX archive...", style("[1/3]").bold().green());
    println!("  Input: {}", args.input.display());
    println!("  Output: {}", args.output.display());
    
    // 1. Write Stub
    let mut out_file = File::create(&args.output).context("Failed to create output file")?;
    out_file.write_all(STUB_BYTES).context("Failed to write stub to output file")?;
    out_file.flush()?; 
    
    let stub_len = out_file.metadata()?.len();
    
    println!("{} Compressing data...", style("[2/3]").bold().green());

    {
        // 2. Setup Compression Stream
        let mut encoder = Encoder::new(&mut out_file, args.level)?;
        
        let mut tar = Builder::new(encoder);
        
        // Add Config file if needed
        if let Some(cmd) = &args.exec {
            let config_content = format!("exec={}", cmd);
            let mut header = Header::new_gnu();
            header.set_size(config_content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, CONFIG_FILENAME, config_content.as_bytes())?;
        }

        // 3. Add files with Smart Path Logic
        let input_path = args.input.canonicalize().unwrap_or(args.input.clone());
        let parent_path = input_path.parent().unwrap_or(&input_path); 
        // If input is "d:\foo\bar", parent is "d:\foo". Relative is "bar/...".
        // If input is "d:\foo\file.txt", parent is "d:\foo". Relative is "file.txt".
        
        // Note: if input is a directory, we walk it.
        // if input is a file, we just add it.
        
        if input_path.is_file() {
            let file_name = input_path.file_name().unwrap();
            let mut f = File::open(&input_path)?;
            tar.append_file(file_name, &mut f).context("Failed to append file")?;
        } else if input_path.is_dir() {
            let walker = WalkDir::new(&input_path);
            for entry in walker {
                let entry = entry?;
                let path = entry.path();
                
                // Smart Relative Path
                let relative_path = path.strip_prefix(parent_path)?;
                
                if relative_path.as_os_str().is_empty() {
                    continue; 
                }

                // If relative path starts with '..', it shouldn't happen due to canonicalize/parent logic,
                // unless we are at root.

                if path.is_dir() {
                    tar.append_dir(relative_path, path).context("Failed to append dir")?;
                } else {
                    let mut f = File::open(path)?;
                    tar.append_file(relative_path, &mut f).context("Failed to append file")?;
                }
            }
        } else {
            return Err(anyhow::anyhow!("Input path does not exist"));
        }
        
        tar.finish().context("Failed to finish tar")?;
        let mut encoder = tar.into_inner()?;
        encoder.finish().context("Failed to finish zstd")?;
    } 

    // 4. Write Footer
    println!("{} Finalizing SFX...", style("[3/3]").bold().green());
    
    let final_len = out_file.metadata()?.len();
    let archive_size = final_len - stub_len;

    out_file.write_all(&archive_size.to_le_bytes())?;
    out_file.write_all(MAGIC)?;

    println!("{} Done! created {}", style("Success!").bold().green(), args.output.display());
    println!("  Stub size: {} bytes", stub_len);
    println!("  Archive size: {} bytes", archive_size);
    println!("  Total size: {} bytes", final_len + 17);

    Ok(())
}