window.addEventListener('DOMContentLoaded', () => {

    const task_add_btn = document.getElementById("add_task") as HTMLButtonElement;
    const habit_add_btn = document.getElementById("add_habit") as HTMLButtonElement;
    const program_add_btn = document.getElementById("add_program") as HTMLButtonElement;

    // program_add_btn.addEventListener('mouse')
    const time_sel = document.getElementById("time_selector") as HTMLDivElement;
    const start_rect = time_sel.getBoundingClientRect();
    const cursor = document.getElementById("cursor") as HTMLDivElement;
    const cursor_rect = cursor.getBoundingClientRect();
    const current_time_elem = document.getElementById("current_time") as HTMLHeadingElement;
    
    set_cursor_deg(cursor, current_time_elem, cursor_rect, start_rect, time_sel, 0);
    cursor.addEventListener('mousedown',  (ev) => drag_cursor(ev, cursor, current_time_elem, cursor_rect, start_rect, time_sel));

    // console.log("HI");
})

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
function drag_cursor(event, cursor, current_time_elem,cursor_rect, start_rect, time_sel) {
    
    is_cursor_down = true;
    window.addEventListener('mousemove', (evt) => {
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
    })

    window.addEventListener('mouseup', (evt) => {
        is_cursor_down = false;
    })
}  


