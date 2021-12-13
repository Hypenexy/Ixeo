<?php
session_start();
echo "var sitesize = " . filesize("index.php") . "\n";
$loadsize = filesize("loaddev.js");// + filesize("load.php");
echo "var loadsize = " . $loadsize . "\n";
if(isset($_SESSION["dark"])){echo "var darksetting = true";}
else{
    echo "var darksetting = false";
}
echo substr(include("loaddev.js"), 0, -1);//using that to fix the mysterious "1"
//maybe try #a67fff instead of the lighter color?