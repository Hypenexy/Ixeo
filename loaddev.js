
var lo = document.createElement("div");
lo.id = 'lo';
lo.innerHTML = "<h1>Midelight</h1> <div id='firstrow'><div id='stats'><h1>Statistics</h1><h2>" + sitesize + " bytes</h2> is the size of the page! <br><h2>" + loadsize + " bytes</h2> you just loaded by scrolling!</div><div id='about'><h1>About</h1>This is a fun hobby driven side project, a search engine. It is competing to be the <b>fastest loading search engine</b>!</div></div><div id='secondrow'><div id='feedback'><h1>Feedback</h1><textarea></textarea></div><div id='options'><h1>Options</h1><label>Dark mode <input id='dark' type='checkbox'></label></div>"
document.body.appendChild(lo);

var darklink;
var dark = document.getElementById("dark")
if(darksetting==true){
    dark.checked = true
    darklink = 'option?dark=false&red='+window.location
}
else{
    darklink = 'option?dark=true&red='+window.location
}

dark.onchange = function() {
    window.location = darklink
}

var lo = document.getElementById("lo")
lo.style = "height:auto;box-sizing: border-box;margin-top: 20%;background:#f9f9f9;border-top: 1px solid #bbb; width: 100%;padding: 20px;border-radius: 8px;opacity:0;transition: 1s;"
setTimeout(function () {
    lo.style.opacity = "1"
}, 100);


var style = document.createElement('style');
style.innerHTML = '::-webkit-scrollbar{display: block}body{-ms-overflow-style:initial;scrollbar-width:initial;}::-webkit-scrollbar{width:12px}::-webkit-scrollbar-thumb{border-radius:8px;background:#bbb}::-webkit-scrollbar-thumb:hover{background:#999}::-webkit-scrollbar-thumb:active{background:#777}#lo h1{font-weight:300;color:#a67fff}#lo{height:100%;box-sizing:border-box;margin-top:20%;background:#f9f9f9;border-top:1px solid #bbb;width:100%;padding:20px;border-radius:8px}#firstrow,#secondrow{display:flex;justify-content:space-around;margin-bottom:50px}#firstrow h1,#secondrow h1{color:#65baff}#firstrow h2,#secondrow h2{margin-bottom:5px}#secondrow textarea{background:none}@media only screen and (max-width:800px){#firstrow,#secondrow{display:block}#firstrow div,#secondrow div{margin-bottom:16px;padding:12px;padding-bottom:30px;border-radius:8px;background:#77777711}}label{cursor:pointer}';
document.head.appendChild(style);


var p = document.getElementsByTagName("p")
p[0].style.transition = "0.5s"
setTimeout(function () {
    p[0].style.opacity = "0.0"
}, 100);
setTimeout(function () {
    p[0].style.display = "none"
}, 5000);