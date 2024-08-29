

type UsualTask = {
    
    id : number,
    genre : string,
    name : string,
    total_spent : number, 
    week_prob : number,
    week_eff : number,
    daytime_prob : number,
    daytime_eff : number,
    
}

let opened_elem : HTMLElement | null = null;
window.addEventListener('DOMContentLoaded',async () => {
    // Add button
    let add_usualtsk = document.getElementById("ustsk_add_button") as HTMLElement;
    add_usualtsk.addEventListener('mousedown', async (_) => {
        
        let inp = document.getElementById("add_ustsk") as HTMLInputElement;
        let [genre, name] = inp.value.split("::");

        let req_body = {
            'genre' : genre,
            'name' : name
        }

        let res = await fetch("http://localhost:3050/create_ustask", {method : "post", body : JSON.stringify(req_body)});
        let t = await res.text();
        if(t == "Ok") 
            window.location.reload()
        
        
    });


    // Set card
    let s_card = document.getElementById("set_card") as HTMLDivElement;
    s_card.addEventListener('mouseup', () => {
        let tasklist = document.getElementById("set_ustask") as HTMLElement;
        
        tasklist.style.display = "flex";
        opened_elem = tasklist;
            

    });
    



    // Adding ALl usual tasks
    let us_tasklist = document.getElementById("usual_tasks") as HTMLUListElement;
    let res = await fetch("http://localhost:3050/usual_tasks", {method : "post"});
    let usual_tasks : Array<UsualTask> = await res.json();
    usual_tasks.forEach((task) => {
        let li_elem = document.createElement('li');
        
        let p_elem = document.createElement('p');
        p_elem.innerText = task.genre + "::" + task.name;
        li_elem.appendChild(p_elem);

        let icons_d = document.createElement("div");
        icons_d.className = 'ed_icons';
        let icon_e = document.createElement('i');
        icon_e.className = 'fa-solid fa-pen';
        let icon_d = document.createElement('i');
        icon_d.className =  'fa-solid fa-x';
        icons_d.appendChild(icon_e);
        icons_d.append(icon_d);
        li_elem.append(icons_d);

        us_tasklist.insertBefore(li_elem, us_tasklist.firstChild);
    })


    document.addEventListener('mouseup', (ev) => {
        let target = ev.target as HTMLElement;
        console.log(ev.target)
        if(ev.target != s_card && target.parentElement != s_card && target.parentElement?.parentElement != s_card && opened_elem != null) {
            opened_elem.style.display = 'none';
            opened_elem = null
        }
    })
    

})