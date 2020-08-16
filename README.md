# SPOT

An HTTP rust crate that emphezies simplicity and minimalism. Inspired by Flask and Exress.js

Features:

- Worker-style multithreading
- Zero dependencies
- \<500 lines of code. Easy to audit and extend
- Easy to use

# Getting started

Here is some example code that shows spot in action. It should be self explanatory if you are familiar with http libraries.

```rust
extern crate spot;
use spot::request::Request;
use spot::response::Response;

fn main() {
    // Number is how many worker threads you want
    let mut app = spot::Spot::new(6);

    app.route("/", |req: Request, mut res: Response| -> Response {
        if req.method == "GET" {
            res.status(200);
            res.body("<h1>Hello World!</h1>");
            return res;
        } else {
            return res;
        };
    });

    app.route("/post", |req: Request, mut res: Response| -> Response {
        println!("{}", req.body);
        if req.method == "POST" {
            res.status(200);
            res.body("{\"message\": \"Hello World!\"}");
            res.header("content-type", "application/json");
            return res;
        } else {
            return res;
        };
    });

    let err = app.bind("127.0.0.1:3000");
    println!("{}", err.unwrap());
}

```
