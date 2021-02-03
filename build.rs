use io::Write;
use std::{env, fs, io, path::PathBuf, process::Command};

const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gresources>
    <gresource prefix="/tk/categulario/pizarra">
"#;

const XML_FOOTER: &str = r#"    </gresource>
</gresources>
"#;

const RESOURCES: [&str; 10] = [
    "alpha.svg",
    "ellipse.svg",
    "eraser.svg",
    "line.svg",
    "pizarra.glade",
    "pizarra.svg",
    "polygon.svg",
    "rectangle.svg",
    "thickness.svg",
    "pizarra.glade",
];

fn main() {
    println!("cargo:rerun-if-changed=res/pizarra.glade");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut resources = PathBuf::from(&out_dir);
    resources.push("res");

    fs::create_dir_all(resources.clone()).unwrap();

    for filename in RESOURCES.iter() {
        let mut filewithpath = PathBuf::from("res/");
        filewithpath.push(filename);
        let mut destfile = resources.clone();
        destfile.push(filename);
        fs::copy(filewithpath, destfile).unwrap();
    }

    let mut xml = String::with_capacity(
        XML_HEADER.len() + XML_FOOTER.len() + RESOURCES.iter().map(|s| s.len()).sum::<usize>()
    );

    xml.push_str(XML_HEADER);

    for filename in RESOURCES.iter() {
        xml.push_str(&format!(
            "\t\t<file>{}</file>\n",
            filename
                .trim_start_matches("res/")
        ));
    }

    xml.push_str(XML_FOOTER);

    let resource_xml = {
        let mut f = resources.clone();
        f.push("resources.xml");
        f
    };
    let mut file = fs::File::create(resource_xml).unwrap();
    file.write_all(xml.as_bytes()).unwrap();

    let mut cmd = Command::new(if let Ok(path) = env::var("GLIB_COMPILE_RESOURCES") {
        path
    } else if cfg!(target_os = "window") {
        "glib-compile-resources.exe".to_owned()
    } else {
        "glib-compile-resources".to_owned()
    });

    cmd.arg("resources.xml")
        .current_dir(resources)
        .output()
        .expect("failed to compile resources");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("wix/pizarra.ico");
        res.compile().unwrap();
    }
}
