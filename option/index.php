<?php
if(!isset($_SESSION)){
    session_start();
}
if(isset($_GET["dark"])){
    if($_GET["dark"]=="false"){
        unset($_SESSION["dark"]);
    }
    else{
        $_SESSION["dark"] = "true";
    }
    echo $_SESSION["dark"];
}
if(isset($_GET["red"])){
    header("Location: " . $_GET["red"]);
}
else{
    header("Location: ../");
}