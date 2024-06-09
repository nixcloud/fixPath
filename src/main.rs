mod fix_path;
use clap::{Arg, ArgAction, Command};
use std::{env, fs, process};
use colored::Colorize;

use object::{LittleEndian, pe};
use object::read::coff::CoffHeader;
use object::read::pe::{ImageNtHeaders};
use object::read::{SectionIndex};

fn main() {
    let matches = Command::new("fixPath")
        .about("fixPath to modify DLL locations in PE executables")
        .arg(
            Arg::new("version")
                .long("version")
                .help("Prints the version")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list-imports")
                .long("list-imports")
                .help("Calls the list_imports function with a filename")
                .value_name("FILENAME")
        )
        .arg(
            Arg::new("set-import")
                .long("set-import")
                .help("Calls the set_import function with two arguments")
                .num_args(2)
                .value_name("KEY VALUE")
                .required(false),
        )
        .group(
            clap::ArgGroup::new("commands")
                .args(&["version", "list-imports", "set-import"])
                .required(true)
                .multiple(false),
        )
        .get_matches();

    if matches.get_flag("version") {
        println!("fixPath version 1.0");
    } else if let Some(filename) = matches.get_one::<String>("list-imports") {
        process_imports(Some(filename), None);
    } else if let Some(values) = matches.get_many::<String>("set-import") {
        let args: Vec<&str> = values.map(|s| s.as_str()).collect();
        process_imports(Some(args[0]), Some(args[1]));
    }
}


fn process_imports(old_import_path: Option<&str>, new_import_path: Option<&str>) {

    let Some(in_file_path) = old_import_path else { todo!() };

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
        object::FileKind::Pe32 => process_file::<pe::ImageNtHeaders32>(in_data),
        object::FileKind::Pe64 => process_file::<pe::ImageNtHeaders64>(in_data),
        _ => {
            eprintln!("Not a PE file");
            process::exit(1);
        }
    };
}

fn process_file<Pe: ImageNtHeaders>(in_data: &[u8]) -> Result<(), object::Error> {

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
    // .fixPath section
    // * [u32] version
    // * [u32] fixPathSize
    // * [u32] idata_name_table_size
    // * [u32] didata_name_table_size
    // * array of string idataNameTable dllname
    // * array of string didataNameTable dllname

    // let fix_path_section: Option<(SectionIndex, &pe::ImageSectionHeader)> = in_sections.enumerate()
    //     .find(|(_, section)| {
    //         let s = String::from_utf8_lossy(&section.name);
    //         if s == ".fixPath" {
    //             // println!("{}", "------- .fixPath ----------".yellow());
    //             // println!("{:0x}", section.pointer_to_raw_data.get(LittleEndian));
    //             true
    //         } else {
    //             false
    //         }
    //     });
    // match fix_path_section {
    //     Some(p) => {
    //         let offset = p.1.pointer_to_raw_data.get(LittleEndian);
    //         println!("found {}", offset);
    //         let fix_path_version = fix_path::read_fix_path_header(&in_data, offset);
    //     },
    //     None => {
    //         println!("No .fixPath section found in PE executable!")
    //     },
    // }

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

    // FIXME do modifications
    // if let Err(err) = fs::write(&out_file_path, out_data) {
    //     eprintln!("Failed to write file '{}': {}", out_file_path, err);
    //     process::exit(1);
    // }

    Ok(())

}
