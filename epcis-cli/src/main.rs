use std::io::{self, Read};
use std::fs;
use clap::Parser;
use epcis_hash::{canonicalize_json, canonicalize_xml, compute_hash_from_prehash};
use epcis_translate::{Sgtin, Sscc, Sgln, Grai, Giai};

#[derive(Parser, Debug)]
#[command(
    author = "Duma Mtungwa <duma@matangaresearch.com>",
    version,
    about = "EPCIS SDK Command Line Interface",
    long_about = "A command line tool to generate canonical pre-hashes, final SHA-256 hashes, and translate GS1 identifiers between URN and Digital Link formats."
)]
struct Args {
    /// File path to parse (use "-" or omit to read from standard input)
    file: Option<String>,

    /// Print the canonical pre-hash string instead of the hash URN
    #[arg(short, long)]
    prehash: bool,

    /// Force input format: "json" or "xml"
    #[arg(short, long)]
    enforce_format: Option<String>,

    /// Translate a GS1 key between EPC URN and Digital Link formats
    #[arg(short, long)]
    translate: Option<String>,

    /// The company prefix length to use for Digital Link translation
    #[arg(short = 'l', long, default_value_t = 7)]
    prefix_len: usize,

    /// Use legacy CBV 1.2 rules (omits user extensions from hashing)
    #[arg(long)]
    cbv_1_2: bool,

    /// The base URL to use when translating URN to Digital Link
    #[arg(long, default_value = "https://id.gs1.org")]
    base_url: String,
}

fn handle_translation(key: &str, prefix_len: usize, base_url: &str) -> Result<String, String> {
    if key.starts_with("urn:epc:id:") {
        // Translate from URN to Digital Link
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() < 5 {
            return Err("Invalid URN format. Expected e.g. urn:epc:id:sgtin:...".to_string());
        }
        let scheme = parts[3];
        match scheme {
            "sgtin" => {
                let sgtin = Sgtin::from_urn(key).map_err(|e| format!("Failed to parse SGTIN: {:?}", e))?;
                Ok(sgtin.to_digital_link(base_url))
            }
            "sscc" => {
                let sscc = Sscc::from_urn(key).map_err(|e| format!("Failed to parse SSCC: {:?}", e))?;
                Ok(sscc.to_digital_link(base_url))
            }
            "sgln" => {
                let sgln = Sgln::from_urn(key).map_err(|e| format!("Failed to parse SGLN: {:?}", e))?;
                Ok(sgln.to_digital_link(base_url))
            }
            "grai" => {
                let grai = Grai::from_urn(key).map_err(|e| format!("Failed to parse GRAI: {:?}", e))?;
                Ok(grai.to_digital_link(base_url))
            }
            "giai" => {
                let giai = Giai::from_urn(key).map_err(|e| format!("Failed to parse GIAI: {:?}", e))?;
                Ok(giai.to_digital_link(base_url))
            }
            other => Err(format!("Unsupported URN scheme: {}", other)),
        }
    } else if key.starts_with("http") || key.contains('/') {
        // Translate from Digital Link to URN
        if key.contains("/01/") {
            let sgtin = Sgtin::from_digital_link(key, prefix_len).map_err(|e| format!("Failed to parse SGTIN DL: {:?}", e))?;
            Ok(sgtin.to_urn())
        } else if key.contains("/00/") {
            let sscc = Sscc::from_digital_link(key, prefix_len).map_err(|e| format!("Failed to parse SSCC DL: {:?}", e))?;
            Ok(sscc.to_urn())
        } else if key.contains("/414/") {
            let sgln = Sgln::from_digital_link(key, prefix_len).map_err(|e| format!("Failed to parse SGLN DL: {:?}", e))?;
            Ok(sgln.to_urn())
        } else if key.contains("/8003/") {
            let grai = Grai::from_digital_link(key, prefix_len).map_err(|e| format!("Failed to parse GRAI DL: {:?}", e))?;
            Ok(grai.to_urn())
        } else if key.contains("/8004/") {
            let giai = Giai::from_digital_link(key, prefix_len).map_err(|e| format!("Failed to parse GIAI DL: {:?}", e))?;
            Ok(giai.to_urn())
        } else {
            Err("Could not detect GS1 Application Identifier (AI) in Digital Link path (expected e.g. /01/ SGTIN, /00/ SSCC, /414/ SGLN, /8003/ GRAI, /8004/ GIAI)".to_string())
        }
    } else {
        Err("Invalid key format. Key must be an EPC URN (starts with urn:epc:id:) or GS1 Digital Link URL/path".to_string())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 1. Handle Key Translation
    if let Some(ref key) = args.translate {
        match handle_translation(key, args.prefix_len, &args.base_url) {
            Ok(translated) => {
                println!("{}", translated);
                return Ok(());
            }
            Err(e) => {
                eprintln!("Translation Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // 2. Read Input Content
    let mut content = String::new();
    if let Some(ref filepath) = args.file {
        if filepath == "-" {
            io::stdin().read_to_string(&mut content)?;
        } else {
            content = fs::read_to_string(filepath)?;
        }
    } else {
        io::stdin().read_to_string(&mut content)?;
    }

    let trimmed = content.trim();
    if trimmed.is_empty() {
        eprintln!("Error: Input is empty.");
        std::process::exit(1);
    }

    // 3. Determine Format
    let is_xml = if let Some(ref format) = args.enforce_format {
        format.to_lowercase() == "xml"
    } else {
        trimmed.starts_with('<')
    };

    let is_cbv_2_0 = !args.cbv_1_2;

    // 4. Generate Pre-hashes
    let prehashes_str = if is_xml {
        canonicalize_xml(&content, is_cbv_2_0)?
    } else {
        let json_val = serde_json::from_str(&content)?;
        canonicalize_json(&json_val, is_cbv_2_0)?
    };

    // 5. Output
    if args.prehash {
        print!("{}", prehashes_str);
    } else {
        // Compute final hashes line by line
        for line in prehashes_str.lines() {
            if !line.trim().is_empty() {
                let hash = compute_hash_from_prehash(line);
                println!("{}", hash);
            }
        }
    }

    Ok(())
}
