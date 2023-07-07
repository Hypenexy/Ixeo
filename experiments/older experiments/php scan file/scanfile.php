<?php
include('../db.php');
$conn = new mysqli($host, $user, $pass);

//$sql = "DROP DATABASE `ixeoExperiment2`";
//if ($conn->query($sql) === TRUE) echo 'Database successfully deleted<br>';

$sql = "CREATE DATABASE `ixeoExperiment2`";
if ($conn->query($sql) === TRUE) echo 'Database successfully created<br>';

$sql = "CREATE TABLE `ixeoExperiment2`.`cachednet` ( `url` VARCHAR(400) NOT NULL , `content` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$sql = "CREATE TABLE `ixeoExperiment2`.`status` ( `indexing` VARCHAR(400) NOT NULL, `indexingnum` VARCHAR(200) NOT NULL, `indexingnummax` VARCHAR(200) NOT NULL, `started` VARCHAR(40) NOT NULL) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';

$conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');

$num = 0;
$agent= 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.93 Safari/537.36';
//error_reporting(0);
ini_set('memory_limit', -1);
ini_set('max_execution_time', '0');
set_time_limit(0);
ini_set('default_socket_timeout', 2);

$time = strtotime("now");
$sql = "INSERT INTO `status` (`indexing`, `indexingnum`, `indexingnummax`, `started`) VALUES ('0', '0', '5037062', '$time')";
if ($conn->query($sql) === TRUE) echo 'Status created<br>';
else{echo $conn->error;}

echo 'Starting scan<br>';

$file = fopen("files/ru_domains", "r");
if ($file) {
    while (($line = fgets($file)) !== false) {
        if(!$conn){
            $conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');
        }
        $url = strtok($line, '.ru') . ".ru";
        $sql = "UPDATE `status` SET `indexing`='$url',`indexingnum`='$num'";
        $conn->query($sql);
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, 5); 
        curl_setopt($ch, CURLOPT_TIMEOUT, 5);
        curl_setopt($ch, CURLOPT_SSL_VERIFYPEER, false);
        curl_setopt($ch, CURLOPT_VERBOSE, true);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
        curl_setopt($ch, CURLOPT_FOLLOWLOCATION, 1);
        curl_setopt($ch, CURLOPT_USERAGENT, $agent);
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        $content = curl_exec($ch);
        if(!empty($content)){
            $urlimage = $url . "/favicon.ico";
            curl_setopt($ch, CURLOPT_URL, $urlimage);
            $image = curl_exec($ch);
            if($image!=$content){
                $urlimagefile = str_replace(["/", ":"], "", $urlimage);
                file_put_contents("images/" . $urlimagefile, $image);
            }
            $content = $conn->real_escape_string($content);
            $sql = "INSERT INTO `cachednet` (`url`, `content`) VALUES ('$url', '$content')";
            mysqli_query($conn, $sql);
            if ($conn->query($sql) !== TRUE){
                $conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');
            }
        }
        curl_close($ch);
        $num++;
    }
    fclose($file);
} else {
    echo "Error opening the file!";
}
$conn->close();