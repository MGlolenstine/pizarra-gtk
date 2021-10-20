use io::Write;
use std::{env, fs, io, path::PathBuf, process::Command};

const XML_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<gresources>
    <gresource prefix="/tk/categulario/pizarra">
"#;

const XML_FOOTER: &str = r#"    </gresource>
</gresources>
"#;

const RESOURCES: &[&str] = &[
    "pizarra.glade",

    "icons/tk.categulario.pizarra.svg",

    "icons/alpha.svg",
    "icons/circle_by_three_points.svg",
    "icons/circle_by_center_and_point.svg",
    "icons/ellipse_by_foci_and_point.svg",
    "icons/eraser.svg",
    "icons/line.svg",
    "icons/polygon.svg",
    "icons/rectangle.svg",
    "icons/thickness.svg",
];

fn main() {
    println!("cargo:rerun-if-changed=res/pizarra.glade");

    let out_dir = env::var("OUT_DIR").unwrap();
    let mut resources = PathBuf::from(&out_dir);
    resources.push("res");

    for filename in RESOURCES.iter() {
        let mut filewithpath = PathBuf::from("res/");
        filewithpath.push(filename);
        let mut destfile = resources.clone();
        destfile.push(filename);
        fs::create_dir_all(destfile.parent().unwrap()).unwrap();
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
    } else if cfg!(target_os = "windows") {
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
        res.set_icon("res/pizarra128.ico");
        res.compile().unwrap();
    }
}
