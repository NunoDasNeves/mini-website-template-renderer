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

    // We'll construct a blog.html with a list of blogs
    // (path of output file, string of the blog)
    let mut blog_list: Vec<(PathBuf, String)> = vec![];

    // traverse child directories of in_dir, doing 3 main things:
    //   - mirror directory structure in out_dir
    //   - convert md files using main template to html files
    //   - copy other files (excluding template.html and blog.html)
    
    // we shouldn't need a seen set - assume no cycles...
    let mut unseen = vec![(in_dir.clone(), out_dir.clone(), false)];

    while unseen.len() > 0 {
        
        let (curr_in_dir, curr_out_dir, in_blogs_dir) = unseen.pop().unwrap();

        for entry in fs::read_dir(curr_in_dir).unwrap() {

            let entry = entry.unwrap();
            let curr_in_path = entry.path();
            let mut filename = entry.file_name().into_string().unwrap();

            // mirror dir structure
            if curr_in_path.is_dir() {
                let mut new_out_dir = curr_out_dir.clone();
                new_out_dir.push(&filename); // filename is a directory name here
                fs::create_dir_all(&new_out_dir).expect("Error creating dir");
                // all child directories of in_dir/blogs should have this as 'true'
                let new_in_blogs_dir = in_blogs_dir || filename == "blogs";
                // add new dir to the list of unseen
                unseen.push((curr_in_path, new_out_dir, new_in_blogs_dir));
                continue
            }

            // convert md files
            else if filename.ends_with(".md") {

                // convert to html
                let md_string = fs::read_to_string(curr_in_path).unwrap();
                let html_string = comrak::markdown_to_html(&md_string, &comrak::ComrakOptions::default());

                // render            
                content_map.insert("content".to_string(), html_string.clone());
                let out_string = handlebars.render("main", &content_map).unwrap();

                // .md to .html
                filename.truncate(filename.len() - 2);
                filename += "html";

                let mut new_out_path = curr_out_dir.clone();
                new_out_path.push(&filename);

                // save to out_dir
                let mut out_file = fs::File::create(&new_out_path).unwrap();
                out_file.write_all(out_string.as_bytes()).unwrap();

                // save the blogs to a list
                if in_blogs_dir {
                    blog_list.push((new_out_path, html_string));
                }
            }

            // move other files to out_dir
            else if filename != "template.html" && filename != "blog.html" {
                let mut new_out_path = curr_out_dir.clone();
                new_out_path.push(&filename);
                fs::copy(curr_in_path, new_out_path).unwrap();
            }
        }
    }

    // Create blogs.html
    let mut blogs_html = String::from("<h1>Recent Blogs</h1>");
    for (fs_path, html_string) in blog_list {
        // convert fs path to anchor href
        let mut link_path = PathBuf::from("/");
        link_path.push(fs_path.strip_prefix(&out_dir).unwrap());
        let link_path = String::from(link_path.to_str().unwrap());
        content_map.insert("blog_link".to_string(), link_path);

        let blog_str = html_string.split("</p>").collect::<Vec<&str>>()[0].to_string() + "</p>";
        content_map.insert("blog_content".to_string(), blog_str);

        let blog_list_html = handlebars.render("blog", &content_map).unwrap();
        blogs_html += &blog_list_html;
    }

    // render with main template
    content_map.insert("content".to_string(), blogs_html);
    let out_string = handlebars.render("main", &content_map).unwrap();

    // save to out_dir
    let mut blogs_path = out_dir.clone();
    blogs_path.push("blogs.html");
    let mut out_file = fs::File::create(&blogs_path).unwrap();
    out_file.write_all(out_string.as_bytes()).unwrap();

}
