window.addEventListener('DOMContentLoaded', () => {
    // Task form setup
    const task_add_btn = document.getElementById("add_task") as HTMLButtonElement;
    task_add_btn.addEventListener('mousedown', () => {
        // Make the the task form appear
        const task_form = document.getElementById("task_form") as HTMLDivElement;
        const blur_container = document.getElementById("blur_container") as HTMLDivElement;
        task_form.style.display = "flex";
        blur_container.style.display = 'block';



        // Set Duration selector
        const time_sel = document.getElementById("time_selector") as HTMLDivElement;
        const start_rect = time_sel.getBoundingClientRect();
        const cursor = document.getElementById("cursor") as HTMLDivElement;
        const cursor_rect = cursor.getBoundingClientRect();
        const current_time_elem = document.getElementById("current_time") as HTMLHeadingElement;
        
        set_cursor_deg(cursor, current_time_elem, cursor_rect, start_rect, time_sel, 0);
        cursor.addEventListener('mousedown',  (ev) => drag_cursor(ev, cursor, current_time_elem, cursor_rect, start_rect, time_sel));


        // Set the close button
        const close_btn = document.getElementById("task_form_x") as HTMLElement;
        close_btn.addEventListener('mousedown', (ev) => {
            // Clearing event listeners
            let new_cursor = cursor.cloneNode(true);
            cursor.parentElement?.replaceChild(new_cursor, cursor);
            
            task_form.style.display = blur_container.style.display = "none";
        });



        // Set the create button
        const main_form = document.getElementById("form_main_tsk") as HTMLFormElement;
        main_form.onsubmit = (ev) => {task_form_submit(ev, main_form)};
    })




    const habit_add_btn = document.getElementById("add_habit") as HTMLButtonElement;
    const program_add_btn = document.getElementById("add_program") as HTMLButtonElement;







})


// ###############################################################################################################################
// form submit events
// Todo {
//     pub id : i64,
//     pub name : String,
//     pub subname : String,
//     pub desc : String,
//     pub state : Vec<i64>,
//     pub refr : i64,
//     pub start_time : i64,
//     pub end_time : i64,
//     pub reward_ref : i64,
//     pub due_time : i64,
//     pub expected_time : i64
// }
type Todo = {
    id : number,
    name : string,
    subname : string,
    desc : string,
    state : Array<number>,
    refr : number,
    start_time : number,
    end_time : number, 
    reward_red : number,
    due_time : number, 
    expected_time : number
}
function task_form_submit(ev : SubmitEvent, form : HTMLFormElement){
    ev.preventDefault();


    let data = new FormData(form);
    
    // due date processing
    let due_dates = data.getAll("due_date");
    data.delete("due_date");
    let due_date = new Date(`${due_dates[0]}:${due_dates[1]}`);
    data.append("due_date", `${Math.floor(due_date.getTime() / 1e3)}`);


    // state processing
    let states = data.getAll("state");
    data.delete("state");
    let state = Array(states);
    data.append("state", `${state}`);


    // Time processing
    let secs = Math.floor((prev_angle * MAX_TIME / (Math.PI * 2)) / 1000);
    data.append('expected_time', `${secs}`);

    // Adding all other fields
    data.append('id', '0');
    data.append('refr', '0');
    data.append('start_time', '0');
    data.append('end_time', '0');
    data.append('reward_ref', '0');



    // JSONify it
    let object = {};
    data.forEach((value, key) => {object[key] = value});
    console.log(JSON.stringify(object));
    
        
    
}



// ###############################################################################################################################
// TASK FORM CURSOR
const COMMON_ANGLES = [0, Math.PI/4, Math.PI / 2, 3 * Math.PI / 4, Math.PI, 3 * Math.PI / 2];
const COMMON_THRESHOLD = Math.PI / 20;
const ACC_COLOR = "#4b9dc9";
let is_within_c_angles = (angle, c_angle) => {return Math.abs(c_angle - angle) < COMMON_THRESHOLD};



const MAX_TIME = 2 * 3600_000; // 2Hours. TODO: This should be read from settings.
const CURSOR_MAIN_PX_DIFF = 12;
function set_cursor_deg(cursor, current_time_elem : HTMLHeadingElement, cursor_rect, start_rect, time_sel, current_deg) {
    time_sel.style.backgroundImage = `conic-gradient(${ACC_COLOR} ${current_deg}rad, #aaa 0)`;
    const x_half = start_rect.x + start_rect.width / 2;
    const y_half = start_rect.y + start_rect.height / 2;

    const actual_cursor_c_width = start_rect.width / 2 + CURSOR_MAIN_PX_DIFF;
    const actual_cursor_c_height = start_rect.height / 2 + CURSOR_MAIN_PX_DIFF;
    cursor.style.left = `${x_half -cursor_rect.width / 2 + actual_cursor_c_width * Math.cos(Math.PI / 2 - current_deg)}px`;
    cursor.style.top = `${y_half - cursor_rect.height / 2 - actual_cursor_c_height * Math.sin(Math.PI / 2 -current_deg)}px`;
    cursor.style.transform = `rotate(${current_deg}rad)`;

    let time = current_deg * MAX_TIME / ( 2 * Math.PI );
    let date = new Date(time);

    let text = "";
    if(time == 0)
        text = "No time";
    else if(time < 3600_000) 
        text = `${date.getMinutes()}min`;
    else if (Number.isInteger(time / 3600_000)) 
        text = `${date.getHours()}hr`;
    else 
        text = `${date.getHours()}h${date.getMinutes()}min`

    current_time_elem.innerText = text;
}



let is_cursor_down = false;
let prev_angle = 0;
let rad_to_deg = (rad) => {return rad * 180 / Math.PI};
function drag_cursor(event, cursor : HTMLDivElement, current_time_elem,cursor_rect, start_rect, time_sel) {
    cursor.style.fontWeight = '800';
    is_cursor_down = true;
    let change_pos = (evt) => {
        if (is_cursor_down)  {
            const x_half = start_rect.x + start_rect.width / 2;
            const y_half = start_rect.y + start_rect.height / 2;
            
            let x_diff = evt.x - x_half;
            let y_diff = evt.y - y_half;

            let angle = 0;
            if (x_diff != 0) {
                angle = Math.atan(y_diff / x_diff);
                if(x_diff <= 0) 
                    angle += Math.PI;
            
                angle += Math.PI/2 // THis cancels out in the set_cursor_deg function yet when i try to remove them both the shit literally lags without explication. So it is there to stay
            } else {
                if (y_diff > 0 )
                    angle = Math.PI;
                else 
                    angle = 0;

            }

            COMMON_ANGLES.forEach((c_angle) => {
                if(is_within_c_angles(angle, c_angle)) {
                    angle = c_angle;
                }
            })

            // Checker not to overlap
            if((prev_angle < Math.PI / 4 && angle > 3 * Math.PI / 2)) 
                return
            else if((prev_angle >= 1.98 * Math.PI && angle < prev_angle * 0.6) ) {
                angle = Math.PI * 2;
                set_cursor_deg(cursor, current_time_elem, cursor_rect, start_rect, time_sel, angle);
            }
            else  {
                set_cursor_deg(cursor, current_time_elem, cursor_rect, start_rect, time_sel, angle);
                prev_angle = angle;
            }
        }
    }
    window.addEventListener('mousemove', change_pos);

    window.addEventListener('mouseup', (evt) => {
        is_cursor_down = false;
        cursor.style.fontWeight = '500';
        window.removeEventListener('mousemove', change_pos);
    })
}  


