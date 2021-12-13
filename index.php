<?php
session_start();
if(isset($_GET["q"])){include("search.php");exit();}
if(isset($_POST["load"])){
    echo "here's more info!";
    exit();
}
?>
<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=0"><title>Ixeo</title><style>*{font-family:'Roboto',sans-serif}::-webkit-scrollbar{width:0px}body{margin:0;margin-top:25vh;text-align:center;height:105vh;-ms-overflow-style:none;scrollbar-width:none}a,p{font-size:84px;font-weight:400;background:linear-gradient(117deg,#65baff 0%,#d0bbff 100%);background-clip:text;-webkit-background-clip:text;-webkit-text-fill-color:transparent}input{width:370px}input,button{border:1px solid #bbb;padding:12px;font-size:16px;border-radius:99px 0 0 99px;outline:0}button{cursor:pointer;font-weight: 300;border-radius:0 99px 99px 0}input:focus,button:focus{border:1px solid #666}@media screen and (max-width: 620px){input{width:60%}}</style></head>
<body>
    <a>Ixeo</a><p style="position:absolute;bottom:0;font-size:24px;width:100%">Swipe up for more</p>
    <form><input autocomplete="off" name="q" placeholder="Search Ixeo or type a URL"><button type="submit">Search</button></form>
    <script>var sc;document.addEventListener('scroll',function(){if(!sc){sc=document.createElement('script');sc.src = "load.php";document.head.appendChild(sc)}}) <?php if(isset($_SESSION['dark'])){echo";var dsc = document.createElement('script');dsc.src = 'dark.js';document.head.appendChild(dsc)";}?></script>
</body>
</html>