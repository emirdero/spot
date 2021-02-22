# SPOT

An HTTP rust crate that emphezies simplicity and minimalism. Inspired by Flask and Exress.js

Features:

- Worker-style multithreading
- Only one dependency
- \<500 lines of code. Easy to audit and extend
- Supports middleware, static file folder
- Easy to use

# Getting started

Here is some example code that shows spot in action. It should be self explanatory if you are familiar with http libraries.

```rust
use spot;
use spot::request::Request;
use spot::response::Response;

fn main() {
    // Create a spot app with 2 worker threads
    let mut app = spot::Spot::new(2);

    // Add a GET endpoint to /
    app.route("/", |req: Request, mut res: Response| -> Response {
        if req.method == "GET" {
            res.status(200);
            res.body("<h1>Hello World!</h1>");
            return res;
        } else {
            // Default response is 404
            return res;
        };
    });

    // Add a POST endpoint to /post
    app.route("/post", |req: Request, mut res: Response| -> Response {
        // Spot does not have JSON serilization built inn, 
        // if you want to parse JSON consider combining spot with serde_json (https://crates.io/crates/serde_json)
        println!("{}", req.body);
        if req.method == "POST" {
            res.status(200);
            res.body("{\"message\": \"Hello World!\"}");
            // HTTP headers can be added like this
            res.header("content-type", "application/json");
            return res;
        } else {
            return res;
        };
    });

    // Bind the spot app to port 3000 on the local IP adress
    let err = app.bind("127.0.0.1:3000");
    println!("{}", err);
}


```
