# SPOT

An HTTP server framework that emphezies simplicity and minimalism. Inspired by Flask and Exress.js

Features:

- Worker-style multithreading
- Zero dependencies
- \<500 lines of code. Easy to audit and extend
- Supports middleware, static file folder
- Easy to use
- No unwraps

# Getting started

Here is some example code that shows spot in action. It should be self explanatory if you are familiar with http libraries.

```rust
extern crate spot;
use spot::request::Request;
use spot::response::Response;

fn main() {
    // Create a spot app with 2 worker threads
    let mut app = spot::Spot::new(2);

    // Use a directory called public in the project root to serve static files
    app.public("public");

    // Middleware for all requests starting with /post/
    app.middle(
        "/post/",
        |req: Request, mut res: Response| -> (Request, Response, bool) {
            if req.method == "POST" {
                if req.body.len() > 0 {
                    return (req, res, true);
                }
                res.status(400);
            }
            return (req, res, false);
        },
    );

    // Redirect GET / to GET /index.html wich is a file in the public directory
    app.route("/", |req: Request, mut res: Response| -> Response {
        if req.method == "GET" {
            res.status(301);
            res.header("Location", "/index.html");
        }
        return res;
    });

    // GET with params
    app.route("/user/", |req: Request, mut res: Response| -> Response {
        let param_keys = ["name", "age"];
        if req.method == "GET" {
            for key in param_keys.iter() {
                if !req.params.contains_key(&key[..]) {
                    res.status(400);
                    res.body(format!("Missing parameter: {}", key));
                    return res;
                }
            }
            res.status(200);
            res.body(format!(
                "<h1>Hello {}, age {}!</h1>",
                req.params.get("name").unwrap(),
                req.params.get("age").unwrap(),
            ));
            return res;
        } else {
            // Default response is 404
            return res;
        };
    });

    // Add a POST endpoint to /post
    app.route("/post/", |req: Request, mut res: Response| -> Response {
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
