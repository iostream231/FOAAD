use backend::webserver::{create_usual_task, get_usual_tasks};
use ntex::web;


#[web::get("/")]
async fn hello() -> impl web::Responder {
    web::HttpResponse::Ok().body("hello, world!")
}



#[ntex::main]
async fn main() -> std::io::Result<()> {
    web::HttpServer::new(|| {
        web::App::new()
            .service(hello)
            .service(create_usual_task)
            .service(get_usual_tasks)
        })
        .workers(4)
        .bind(("127.0.0.1", 3050))?
        .run()
        
        .await
    
}