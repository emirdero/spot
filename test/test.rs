use spot;
use spot::request::Request;
use spot::response::Response;

fn main() {
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
