mod fix_path;// use std::error::Error;
use std::{env, fs, process};
// use std::collections::HashSet;
use colored::Colorize;

use object::{LittleEndian, pe};
use object::read::coff::CoffHeader;
use object::read::pe::{ImageNtHeaders};
// use object::LittleEndian as LE;

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        eprintln!("Usage: {} <infile>", args.next().unwrap());
        process::exit(1);
    }

    args.next();
    let in_file_path = args.next().unwrap();

    let in_file = match fs::File::open(&in_file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to open file '{}': {}", in_file_path, err,);
            process::exit(1);
        }
    };
    let in_data = match unsafe { memmap2::Mmap::map(&in_file) } {
        Ok(mmap) => mmap,
        Err(err) => {
            eprintln!("Failed to map file '{}': {}", in_file_path, err,);
            process::exit(1);
        }
    };
    let in_data = &*in_data;

    let kind = match object::FileKind::parse(in_data) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to parse file: {}", err);
            process::exit(1);
        }
    };
    let _out_data = match kind {
        object::FileKind::Pe32 => fix_file::<pe::ImageNtHeaders32>(in_data),
        object::FileKind::Pe64 => fix_file::<pe::ImageNtHeaders64>(in_data),
        _ => {
            eprintln!("Not a PE file");
            process::exit(1);
        }
    };
}

fn fix_file<Pe: ImageNtHeaders>(in_data: &[u8]) -> Result<(), object::Error> {

    // println!(
    //     "{}, {}, {}, {}, {}, {}, and some normal text.",
    //     "Bold".bold(),
    //     "Red".red(),
    //     "Yellow".yellow(),
    //     "Green Strikethrough".green().strikethrough(),
    //     "Blue Underline".blue().underline(),
    //     "Purple Italics".purple().italic()
    // );

    let in_dos_header = pe::ImageDosHeader::parse(in_data)?;
    let mut nt_headers_offset = in_dos_header.nt_headers_offset().into();
    // let in_rich_header = object::read::pe::RichHeaderInfo::parse(in_data, offset);
    let (in_nt_headers, in_data_directories) = Pe::parse(in_data, &mut nt_headers_offset)?;
    let in_file_header = in_nt_headers.file_header();
    // let in_optional_header = in_nt_headers.optional_header();
    let in_sections = in_file_header.sections(in_data, nt_headers_offset)?;

    let import_table = in_data_directories.import_table(in_data, &in_sections)?.unwrap();
    let mut import_descriptor_iterator = import_table.descriptors()?;

    /// # generate fixPath records
    let fixPathSection = in_sections.enumerate()
        .find(|(uu, section)| {
            // println!("{}", uu);
            let s = String::from_utf8_lossy(&section.name);
            // println!("{}", s);
            if s == ".fixPath" {
                println!("{}", "------- .fixPath ----------".yellow());
                println!("{:0x}", section.pointer_to_raw_data.get(LittleEndian));
                true
            } else {
                false
            }

        });
    match fixPathSection {
        Some(fix_path) => println!("found"),
        None => println!("not found"),
    }

    // let fix_path_version = fix_path::read_version(&in_data);
    // println!("{}", fix_path_version);

    /// # read **dllName records**

    while let Some(import) = import_descriptor_iterator.next().unwrap() {
        let dll_name_address: u32 = import.name.get(LittleEndian); // e74
        let dll_name_abs_address =import_table.name_address(dll_name_address) + import_table.section_offset();
        let dll_name = std::str::from_utf8(import_table.name(dll_name_address)?);
        match dll_name {
            Ok(s) => {
                println!("- (xxx) -> '{}' @ 0x{:0x}", s, dll_name_abs_address);
            },
            Err(_) => {}
        }
    }


    /// # read delayed dllName records
    let delayed_import_table = in_data_directories.delay_load_import_table(in_data, &in_sections)?.unwrap();
    let mut delayed_import_descriptor_iterator = delayed_import_table.descriptors()?;
    while let Some(delayed_import) = delayed_import_descriptor_iterator.next().unwrap() {
        //println!("{:?}", import);
        let dll_name_address: u32 = delayed_import.dll_name_rva.get(LittleEndian);
        let dll_name_abs_address =import_table.name_address(dll_name_address) + import_table.section_offset();

        let dll_name = std::str::from_utf8(delayed_import_table.name(dll_name_address)?);

        match dll_name {
            Ok(s) => {
                println!("- (xxx) -> '{}' @ 0x{:0x}", s, dll_name_abs_address);
            },
            Err(_) => {}
        }
    }




    // FIXME get .fixPath section
    // extract the
    // * version
    // * fixPathSize
    // * idataNameTable_size
    // * array of string idataNameTable dllname
    // * didataNameTable_size
    // * array of string didataNameTable dllname

    // FIXME do modifications

    // if let Err(err) = fs::write(&out_file_path, out_data) {
    //     eprintln!("Failed to write file '{}': {}", out_file_path, err);
    //     process::exit(1);
    // }

    Ok(())

}
