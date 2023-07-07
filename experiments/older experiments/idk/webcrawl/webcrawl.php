<?php
include('../../app/server/db.php');
$conn = new mysqli($host, $user, $pass, 'ixeo');

$StartPage = "https://www.youtube.com/watch?v=HL8--wNk19Y";

$agent= 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.93 Safari/537.36';
//error_reporting(0);
ini_set('memory_limit', -1);
ini_set('max_execution_time', '0');
set_time_limit(0);
ini_set('default_socket_timeout', 2);

$time = strtotime("now");
$sql = "INSERT INTO `status` (`indexing`, `started`) VALUES ('0', '$time')";
if ($conn->query($sql) === TRUE) echo 'Status created<br>';
else{echo $conn->error;}

echo 'Starting scrape on '.$StartPage.'<br>';

$domain = explode('/', $StartPage)[2];
$url = $StartPage;
$sql = "UPDATE `status` SET `indexing`='$url'";
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
    $urlimage = $domain . "/favicon.ico";
    curl_setopt($ch, CURLOPT_URL, $urlimage);
    $image = curl_exec($ch);
    if($image!=$content){
        //$urlimagefile = str_replace(["/", ":"], "", $urlimage) . '.';
        $urlimagefile = $domain . ".favicon.ico";
        file_put_contents("../../content/favicons/" . $urlimagefile, $image);
    }
    $contentstore = $conn->real_escape_string($content);
    $sql = "INSERT INTO `cachednet` (`url`, `content`) VALUES ('$url', '$contentstore')";
    mysqli_query($conn, $sql);
    $dom = new DOMDocument;

    @$dom->loadHTML($content);

    $links = $dom->getElementsByTagName('a');
    $images = $dom->getElementsByTagName('img');

    foreach ($links as $link){
        $flink = $link->getAttribute('href');
        if($flink != "#"){
            if($flink[0] == "/"){
                echo $url . $flink . '<br>';
            }
            else{
                echo $flink . '<br>';
            }
        }
    }
    
    foreach ($images as $fimage){
        echo $fimage->getAttribute('src') . ' - ' . $fimage->getAttribute('title') . ' - ' . $fimage->getAttribute('alt') . '<br>';
    }
}
curl_close($ch);

echo 'Scrape end<br>';