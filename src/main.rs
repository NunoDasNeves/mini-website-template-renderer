use std::collections::BTreeMap;
use handlebars;
use handlebars::Handlebars;
use comrak;

use std::fs;
use std::io::Write;
use std::env;
use std::process;

use std::path::PathBuf;

fn main() {

    // Parse args
    let args: Vec<String> = env::args().collect();
    let usage = format!("USAGE: {} in_dir out_dir", &args[0]);

    if args.len() != 3 {
        println!("{}", usage);
        process::exit(1);
    }

    let in_dir = PathBuf::from(&args[1]);
    let out_dir = PathBuf::from(&args[2]);

    // open templates and read to string
    let mut template_path = in_dir.clone();
    template_path.push("template.html");
    let template_html = fs::read_to_string(&template_path).expect("Error reading template.html");
    let mut blog_template_path = in_dir.clone();
    blog_template_path.push("blog.html");
    let blog_template_html = fs::read_to_string(&blog_template_path).expect("Error reading blog.html");

    // out dir creation
    fs::create_dir_all(&out_dir).expect("Error creating out_dir");

    // create the handlebars registry
    let mut handlebars = Handlebars::new();
    // turn off escaping
    handlebars.register_escape_fn(handlebars::no_escape);

    // register the template. The template string will be verified and compiled.
    handlebars.register_template_string("main", template_html).expect("Main template invalid!");
    handlebars.register_template_string("blog", blog_template_html).expect("Blog template invalid!");

    // The data type should implements `serde::Serialize`
    let mut content_map = BTreeMap::new();

    // traverse child directories of in_dir, doing 3 things:
    //   - mirror directory structure in out_dir
    //   - convert md files using main template to html files
    //   - copy other files (excluding template.html and blog.html)
    
    let mut unseen = vec![(in_dir.clone(), out_dir.clone())];

    while unseen.len() > 0 {
        
        let (curr_in_dir, curr_out_dir) = unseen.pop().unwrap();

        for entry in fs::read_dir(curr_in_dir).unwrap() {

            let entry = entry.unwrap();
            let curr_in_path = entry.path();
            let mut filename = entry.file_name().into_string().unwrap();

            // mirror dir structure
            if curr_in_path.is_dir() {
                let mut new_out_dir = curr_out_dir.clone();
                new_out_dir.push(&filename); // filename is a directory name here
                fs::create_dir_all(&new_out_dir).expect("Error creating dir");
                // add new dir to the list of unseen
                unseen.push((curr_in_path, new_out_dir));
                continue
            }

            // convert md files
            else if filename.ends_with(".md") {

                // convert to html
                let md_string = fs::read_to_string(curr_in_path).unwrap();
                let html_string = comrak::markdown_to_html(&md_string, &comrak::ComrakOptions::default());

                // render            
                content_map.insert("content".to_string(), html_string);
                let out_string = handlebars.render("main", &content_map).unwrap();

                // .md to .html
                filename.truncate(filename.len() - 2);
                filename += "html";

                let mut new_out_path = curr_out_dir.clone();
                new_out_path.push(&filename);

                // save to out_dir
                let mut out_file = fs::File::create(new_out_path).unwrap();
                out_file.write_all(out_string.as_bytes()).unwrap();
            }

            // move other files to out_dir
            else if filename != "template.html" && filename != "blog.html" {
                let mut new_out_path = curr_out_dir.clone();
                new_out_path.push(&filename);
                fs::copy(curr_in_path, new_out_path).unwrap();
            }
        }
    }


}
