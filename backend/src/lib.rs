pub mod database {
    use homedir::my_home;
    use ntex::web;
    use serde::{Deserialize, Serialize};
    use sqlite::State;
    use std::io::ErrorKind::NotFound;
    use serde_json;


    #[derive(Debug)]
    pub enum DatabaseError {
        SqliteError { e: sqlite::Error },

        InvalidFields,
    }

    impl DatabaseError {
        pub fn get_web_error(&self) -> web::HttpResponse {
            match self {
                Self::InvalidFields => {
                    eprintln!("Invalid fields error");
                    web::HttpResponse::BadRequest().message_body("Invalid fields".into())
                },
                Self::SqliteError { e } => { 
                    eprintln!("Sqlite error : {}", e.message.clone().unwrap());
                    web::HttpResponse::InternalServerError().message_body(e.to_string().into()) }
            }
        }
    }

    fn get_connection() -> sqlite::Connection {
        const DATABASE_MAIN_FILE : &str = "foaad.db";
        const APPLICATION_DIR : &str = "FOAAD";
        let home_path = my_home().unwrap().unwrap();
        let app_path = home_path.join(APPLICATION_DIR);
        let main_db_path = app_path.join(DATABASE_MAIN_FILE);

        let _ = std::fs::read(&main_db_path).unwrap_or_else(|err|  {
            match err.kind() {
                NotFound => { 
                    if let Err(dir_err) = std::fs::create_dir(&app_path) {
                        eprintln!("Error creating dir {} : {}", &app_path.display(), dir_err);
                    }
                    else if let Err(write_err) = std::fs::write(&main_db_path, "") {
                        eprintln!("Error writing to {} : {}", &main_db_path.display(), write_err);
                    }; 
                    Vec::new() },
                _ => { Vec::new() }
            }
        });

        sqlite::open(main_db_path).unwrap()
    }

    pub fn init_db() -> Result<(), sqlite::Error> {
        let cnt = get_connection();
        let query = "SELECT * FROM tasks LIMIT 1";
        if let Ok(_) =  cnt.execute(query) {

            return Ok(());
        } else {
            let init_query = "
                CREATE TABLE planned(name TEXT, subname TEXT, desc TEXT, state TEXT, ref INTEGER, start_time INTEGER, end_time INTEGER, due_time INTEGER, expected_time INTEGER, importance INTEGER);
                CREATE TABLE usual_tasks(genre TEXT, name TEXT, total_spent INTEGER, week_prob TEXT, week_eff TEXT, daytime_prob TEXT, daytime_eff TEXT);
                CREATE TABLE habits(genre TEXT, name TEXT, start_time INTEGER, end_time INTEGER, overall_consistency REAL, week_consistency REAL, usual_task_ref INTEGER);
                CREATE TABLE leisures(name TEXT, process_name TEXT, total_spent INTEGER, week_prob TEXT, daytime_prob TEXT, reward_ref INTEGER);
                CREATE TABLE projects(name INTEGER, tasks_due TEXT, habits TEXT, milestones TEXT);
            ";
            // usual_tasks -> week_{prob|eff} | daytime{prob|eff} are JSON texts containing arrays of each day of the week. see https://www.sqlite.org/json1.html
            // Same for leisure->week_prob|daytime_prob 
            // projects->tasks_due is a JSON text containing array of each task + it's due time
            // projects->habits is a JSON array containig habits refids
            // projects->milestones is a JSON object array of each milestone, it's time and it's result

            cnt.execute(init_query)
        }
    }

    pub fn reset_db() -> Result<(), sqlite::Error> {
        let cnt = get_connection();
        let query = "
            DROP TABLE planned;
            DROP TABLE usual_tasks;
            DROP TABLE habits;
            DROP TABLE leisures;
            DROP TABLE projects";
        let _ = cnt.execute(query); // We don't care if these tables exist or not

        init_db() // Re-init db

    }



    pub struct Leisure {
        id : i64,
        name : String,
        pname : String,
        total_spent : i64,
        week_prob : Vec<f64>,
        daytime_prob : Vec<f64>
    }

    impl Leisure {
        // Get all info about all leisures
        pub fn get_all() -> Vec<Leisure> {
            let mut res: Vec<Leisure> = Vec::new();
            let cnt = get_connection();
            let query = "SELECT rowid,* FROM leisures";
            let mut statement = cnt.prepare(query).unwrap();

            while let Ok(sqlite::State::Row) = statement.next() {
                let n_leisure = Leisure {
                    id : statement.read::<i64, _>("rowid").unwrap(),
                    name : statement.read::<String, _>("name").unwrap(),
                    pname : statement.read::<String, _>("process_name").unwrap(),
                    total_spent : statement.read::<i64, _>("total_spent").unwrap(),
                    week_prob : serde_json::from_str(
                        &statement.read::<String, _>("week_prob").unwrap()
                    ).unwrap(),
                    daytime_prob : serde_json::from_str(
                        &statement.read::<String, _>("daytime_prob").unwrap()
                    ).unwrap()
                };
                res.push(n_leisure);
            };
            res
        }

        // Get only pnames following an index
        pub fn get_indexed_pnames() -> Vec<String> {
            let mut res : Vec<String> = Vec::new();
            
            let cnt = get_connection();
            let query = "SELECT process_name FROM leisures";
            let mut statement = cnt.prepare(query).unwrap();
            while let Ok(State::Row) = statement.next() {
                res.push(
                    statement.read::<String, _>("process_name").unwrap()
                );
            };
            res
        }

        
    }


    
    #[derive(Deserialize, Serialize)]
    pub struct UsualTask {
        pub id : i64,
        pub genre : String,
        pub name : String,
        pub total_spent : i64, 
        pub week_prob : f64,
        pub week_eff : f64,
        pub daytime_prob : f64,
        pub daytime_eff : f64,
    }

    impl UsualTask {
        pub fn create(genre : &str, name : &str) -> Result<(), DatabaseError> {
            let cnt = get_connection();
            let query = format!("INSERT into usual_tasks(genre, name, total_spent, week_prob, week_eff, daytime_prob, daytime_eff) VALUES('{}', '{}', 0, 0, 0, 0, 0)", genre, name);

            let n = cnt.execute(query);

            match n {
                Ok(_) => Ok(()),
                Err(er) => Err(
                    DatabaseError::SqliteError{ e : er}
                )
            }
            
        }

        pub fn get_all() -> Vec<UsualTask> {
            let mut res : Vec<UsualTask> = vec![];

            let cnt = get_connection();
            let query = "SELECT rowid,* FROM usual_tasks";
            let mut statement = cnt.prepare(query).unwrap();

            while let Ok(State::Row) = statement.next() {
                let new = UsualTask {
                    id : statement.read::<i64, _>("rowid").unwrap(),
                    genre : statement.read::<String, _>("genre").unwrap(),
                    name : statement.read::<String, _>("name").unwrap(),
                    total_spent : statement.read::<i64, _>("total_spent").unwrap(),
                    week_prob : statement.read::<f64, _>("week_prob").unwrap(),
                    week_eff : statement.read::<f64, _>("week_eff").unwrap(),
                    daytime_prob : statement.read::<f64, _>("daytime_prob").unwrap(),
                    daytime_eff : statement.read::<f64, _>("daytime_eff").unwrap(),

                };
                res.push(new);
            }

            res
        }
    }


    #[derive(Deserialize, Serialize)]
    pub struct Todo {
        pub id : i64,
        pub name : String,
        pub subname : String,
        pub desc : String,
        pub state : Vec<i64>,
        pub refr : i64,
        pub start_time : i64,
        pub end_time : i64,
        pub reward_ref : i64,
        pub due_time : i64,
        pub expected_time : i64,
        pub importance : i64,
    }


    
    pub enum CrState {
        ACTIVE,
        UNPLANNED,
        DFOCUS,
        LEISURE,
        PFOCUS(Vec<i64>),
    }

    impl Todo {

        // Commits tasks that are due at some point to the database
        pub fn commit_due(&self) -> Result<(), DatabaseError>{
            // Verify values
            if self.name.len() == 0 || self.desc.len() == 0 || self.state.len() == 0 || self.expected_time == 0 {
                return Err(DatabaseError::InvalidFields);
            }

            
            // Insert values
            let cnt = get_connection();
            let query = format!("INSERT INTO planned(name, subname, desc, state, ref, due_time, expected_time, importance) VALUES ('{}', '{}', '{}', json_array('{}'), {}, {}, {}, {})",
                    self.name, self.subname, self.desc, serde_json::to_string(&self.state).unwrap() , self.refr, self.due_time, self.expected_time, self.importance);
            if let Err(sql_err) = cnt.execute(query) {
                return Err(DatabaseError::SqliteError { e : sql_err });
            }


            Ok(())
        }        



        // Highly un-optimized
        pub fn get_current_state(time : i64) -> CrState {
            let cnt = get_connection();


            let query = "SELECT state FROM planned WHERE start_time < ? < end_time";
            let mut statement = cnt.prepare(query).unwrap();
            statement.bind((1, time)).unwrap();

            if let Ok(State::Row) = statement.next() {
                let res : Vec<i64> = serde_json::from_str(&statement.read::<String, _>("state").unwrap()).unwrap();

                match res[0] {
                    0 => { CrState::ACTIVE },
                    2 => { CrState::DFOCUS },
                    3 => { CrState::LEISURE },
                    _ => {CrState::PFOCUS(res)}
                }
            } else {
                CrState::UNPLANNED
            }
        }



        // Gets the current tab
        pub fn get_current(time : i64) -> Vec<Todo> {
            let cnt = get_connection();
            let mut res: Vec<Todo> = Vec::new();

            let query = "SELECT rowid,* FROM planned WHERE start_time < ? < end_time";

            let mut statement = cnt.prepare(query).unwrap();
            statement.bind((1, time)).unwrap();
            while let Ok(State::Row) = statement.next() {
                let todo = Todo {
                    id : statement.read::<i64, _>("id").unwrap(),
                    name : statement.read::<String, _>("name").unwrap(),
                    subname : statement.read::<String, _>("subname").unwrap(),
                    desc : statement.read::<String, _>("desc").unwrap(),
                    state : serde_json::from_str(
                        &statement.read::<String, _>("state").unwrap()
                    ).unwrap(),
                    refr : statement.read::<i64, _>("ref").unwrap(),
                    start_time : statement.read::<i64, _>("start_time").unwrap(),
                    end_time : statement.read::<i64, _>("end_time").unwrap(),
                    reward_ref : statement.read::<i64, _>("reward_ref").unwrap(),
                    due_time : statement.read::<i64, _>("due_time").unwrap(),
                    expected_time : statement.read::<i64, _>("expected_time").unwrap(),
                    importance : statement.read::<i64, _>("importance").unwrap(),
                };

                res.push(todo);
            }

            res
        }



        

        // Gets tasks due after current_time
        pub fn get_tasks_due(current_time : i64) -> Vec<Todo> {
            let cnt = get_connection();
            let query = "SELECT rowid,name,subname,desc,ref,due_time,expected_time,importance FROM planned WHERE due_time > ? AND start_time is NULL ORDER BY due_time ASC LIMIT 5";
            let mut statement = cnt.prepare(query).unwrap();
            statement.bind((1, current_time)).unwrap();

            let mut res : Vec<Todo> = Vec::new();
            while let Ok(State::Row) = statement.next() {
                let todo = Todo {
                    id : statement.read::<i64, _>("rowid").unwrap(),
                    name : statement.read::<String, _>("name").unwrap(),
                    subname : statement.read::<String, _>("subname").unwrap(),
                    desc : statement.read::<String, _>("desc").unwrap(),
                    state : Vec::new(),
                    refr : statement.read::<i64, _>("ref").unwrap(),
                    start_time : 0,
                    end_time : 0,
                    reward_ref : 0,
                    due_time : statement.read::<i64, _>("due_time").unwrap(),
                    expected_time : statement.read::<i64, _>("expected_time").unwrap(),
                    importance : statement.read::<i64, _>("importance").unwrap()
                };

                res.push(todo)
                
            } 

            res
            
        }

    }

}


pub mod webserver {
    use ntex::{http::header::ACCESS_CONTROL_ALLOW_ORIGIN, web};
    use serde::Deserialize;
    use crate::database::UsualTask;

    

    #[web::post("/usual_tasks")]
    async fn get_usual_tasks() -> web::HttpResponse {
        let us_tasks = UsualTask::get_all();


        web::HttpResponse::Ok()
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .json(&us_tasks)

    }


    #[derive(Deserialize)]
    struct CreatedUTask {
        name : String,
        genre : String,
    }
    
    #[web::post("/create_ustask")]
    async fn create_usual_task(usual_task : String) -> web::HttpResponse {
        let usual_task : CreatedUTask = serde_json::from_str(&usual_task).unwrap();
        let res = UsualTask::create(&usual_task.genre, &usual_task.name);

        match res {
            Ok(_) => web::HttpResponse::Ok()
                    .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .message_body("Ok".into()),
            
            Err(e) => e.get_web_error()
        }
        
    }







}



pub mod background_run {
    use std::collections::HashSet;
    use std::path::Path;
    use std::thread;
    use std::process;
    use std::time::SystemTime;


    use crate::database::CrState;
    use crate::database::{Leisure, Todo};

    use tasklist;
    use winrt_notification::Duration;
    use winrt_notification::IconCrop;
    use winrt_notification::Toast;

    fn get_processes() -> Vec<String> {
        let mut res : Vec<String> = Vec::new();
        unsafe {
            let tl = tasklist::Tasklist::new();
            for i in tl {
                res.push(
                    i.get_pname().trim_end_matches(char::from(0)).to_string()
                );
            }
        }
        res
    }


    // Notify Function
    pub fn notify(hero : Option<&Path>,icon : &Path, title : &'static str, text : &'static str, text2 : Option<String>, image : Option<&Path>) {
        let mut new_toast = Toast::new("com.foaad.dev")
            .icon(icon, IconCrop::Circular, "icon")
            .title(&title)
            .text1(&text)
            .duration(Duration::Short);

        if let Some(image_p) = image {
            new_toast = new_toast.image(image_p, "image");
        }
        if let Some(hero_p) = hero {
            new_toast = new_toast.hero(hero_p, "hero");
        }
        if let Some(text2_c) = text2 {
            new_toast = new_toast.text2(&text2_c)
        } 

        new_toast.show().expect("Error writing notification");
    }

    // get current time
    pub fn get_current_time() -> i64{
        let current_time = SystemTime::now()
                                            .duration_since(SystemTime::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                            .try_into()
                                            .unwrap();
        current_time
    }


    // Main background functions
    pub fn background_main() {
        let pnames = Leisure::get_indexed_pnames();

        thread::spawn(move || {
            let mut last_time_a_notif = 0;
            loop {
                let current_time = get_current_time();
                let current_state = Todo::get_current_state(
                    current_time
                );

                match current_state {
                    CrState::DFOCUS => { focus_enforce(pnames.clone(), Vec::new())},
                    CrState::PFOCUS(all) => { 
                        focus_enforce(pnames.clone(),all
                            .iter()
                            .map(|i| (i - 2).try_into().unwrap())
                            .collect())
                    },
                    CrState::UNPLANNED => {

                        let rec = Todo::get_tasks_due(
                            current_time
                        );

                        let rec_vstr : String = rec.iter().map(|e| {
                            format!("• {} : {}\n", e.subname, e.desc)
                        }).collect();
                        

                        notify(None, &Path::new("./icons/icon.png"), 
                            "Plan your day !",
                            "You have no planned for the current time. pls consider you being taxed twice the amount.\nHere are some recommendations : ",
                            Some(rec_vstr),
                            None);
                    },
                    CrState::ACTIVE => {
                        if last_time_a_notif == 0 || last_time_a_notif >= 60 {
                            let rec = Todo::get_tasks_due(
                                current_time
                            );

                            let rec_vstr : String = rec.iter().map(|e| {
                                format!("• {} : {}\n", e.subname, e.desc)
                            }).collect();

                            notify(None, &Path::new("./icons/icon.png"), 
                            "Suggestions : ",
                             "Seems like you are free for now ? Here are some suggestions for you : ",
                            Some(rec_vstr), 
                            None);
                            last_time_a_notif = 1;
                        } else {
                            last_time_a_notif += 1;
                        }
                    },
                    CrState::LEISURE => { /* Do nothing :) */}
                    
                }

            }
        });
    }

    // Enforcer function :D
    // Also unoptimized af
    pub fn focus_enforce(pnames : Vec<String>, allowed : Vec<usize>) {
        let processes : HashSet<_> = get_processes().into_iter().collect();
        let allowed : Vec<String> = allowed.iter().map(|&i| pnames[i].clone()).collect();
        let pnames : HashSet<String> = pnames.into_iter().collect();

        let intersects : Vec<&String> = processes.intersection(&pnames).collect();
        for i in intersects {
            if !allowed.contains(i) {
                process::Command::new("taskkill")
                    .arg("-f")
                    .arg("-im")
                    .arg(i)
                    .output()
                    .expect("Failed to kill task");
            }
        }



    }
}


#[cfg(test)]
mod test {
    use homedir::my_home;
    use winrt_notification::{Duration, IconCrop, Sound, Toast};
    use std::path::Path;

    use crate::{background_run::get_current_time, database::{reset_db, Todo}};
    
    // use super::background_run::get_processes;
    // use crate::database::get_connection; 


    #[test]
    fn test_home_dir() {
        assert_eq!(
            Some(std::path::PathBuf::from("C:\\Users\\pasteb".to_owned())),
            my_home().unwrap()
        )
    }

    // #[test]
    // fn test_access_db() {
    //     get_connection();
    // }

    // #[test]
    // fn test_current_processes() {
        // let processes = get_processes();
        // assert!( processes.contains(&String::from("brave.exe")));
    // }

    
    
    #[test]
    fn test_notification() {

        Toast::new("com.foaad.dev")
            .title("Welcome bro")
            .hero(&Path::new("C:\\Users\\pasteb\\Documents\\Projects\\Time_mgmt\\FOAAD\\src-tauri\\icons\\icon.png"), "FOAAD")
            .icon(&Path::new("C:\\Users\\pasteb\\Documents\\Projects\\Time_mgmt\\FOAAD\\src-tauri\\icons\\icon.png"), IconCrop::Circular, "FOAAD")
            .text1("How are you there :?")
            .text2("I hope you are fine")
            .image(&Path::new("C:\\Users\\pasteb\\Documents\\Projects\\Time_mgmt\\FOAAD\\src-tauri\\icons\\icon.png"), "FOAAD")
            .image(&Path::new("C:\\Users\\pasteb\\Documents\\Projects\\Time_mgmt\\FOAAD\\src-tauri\\icons\\icon.png"), "FOAAD")
            .sound(Some(Sound::SMS))
            .duration(Duration::Short)
            .show()
            .unwrap();
    }


    #[test]
    fn test_reset_db() {
        reset_db().unwrap();
    }

    #[test]
    fn test_commit() {
        reset_db().unwrap();
        let current_time = get_current_time();
        let new_planned = Todo {
            id : 0,
            name : String::from("Test"),
            subname : String::new(),
            desc : String::from("test"),
            state : vec![0],
            refr : 0,
            start_time : 0, 
            end_time : 0,
            reward_ref : 0,
            due_time : current_time + 1800,
            expected_time : 3600,
            importance : 0
        };


        new_planned.commit_due().unwrap();

        let due = Todo::get_tasks_due(current_time);
        assert_eq!(due[0].desc, String::from("test"));

        
    }
    
}