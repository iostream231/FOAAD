window.addEventListener("DOMContentLoaded", () => {
  let nav_lnks = document.getElementsByClassName("nv_link") as HTMLCollectionOf<HTMLAnchorElement>;
  let main_frame = document.getElementById("main_frame") as HTMLIFrameElement;
  for(let i = 0; i < nav_lnks.length; ++i) {
    nav_lnks[i].addEventListener('click', (ev : MouseEvent) => {
      ev.preventDefault();
      let target = ev.target as HTMLAnchorElement;
      if(target.id) 
        main_frame.src = `./${target.id}/index.html`;
      else if (target.parentElement != undefined && target.parentElement.id) 
        main_frame.src = `./${target.parentElement.id}/index.html`;
      else if (target.parentElement?.parentElement != undefined && target.parentElement.parentElement.id)
        main_frame.src = `./${target.parentElement?.parentElement.id}/index.html`;
      else 
        alert("Error opening the wanted window :(");
    })
  }
})