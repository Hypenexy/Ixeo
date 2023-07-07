<?php
include('../db.php');
$conn = new mysqli($host, $user, $pass, 'ixeoexperiment1');
$num = 0;
error_reporting(0);
ini_set('memory_limit', -1);
ini_set('max_execution_time', '86400');
ini_set('default_socket_timeout', 2);

$time = strtotime("now");
$sql = "INSERT INTO `status` (`indexing`, `indexingnum`, `indexingnummax`, `started`) VALUES ('0', '0', '4228250625', '$time')";
$conn->query($sql);
for ($i=1; $i < 255; $i++) { 
    for ($o=1; $o < 255; $o++) { 
        for ($p=1; $p < 255; $p++) { 
            for ($l=1; $l < 255; $l++) {
                $urlshort = $i . "." . $o . "." . $p . "." . $l;
                $url = "http://" .  $i . "." . $o . "." . $p . "." . $l;
                $urls = "https://" .  $i . "." . $o . "." . $p . "." . $l;
                $num++;
                $sql = "UPDATE `status` SET `indexing`='$url',`indexingnum`='$num'";
                $conn->query($sql);
                $ch = curl_init();
                $chs = curl_init();
                curl_setopt($ch, CURLOPT_URL, $url);
                curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
                curl_setopt($chs, CURLOPT_URL, $urls);
                curl_setopt($chs, CURLOPT_RETURNTRANSFER, 1);
                $content = curl_exec($ch);
                $contents = curl_exec($chs);
                curl_close($ch);
                curl_close($chs);
                if(!empty($content)){
                    $contentsecure = $conn->real_escape_string($content);
                    $sql = "INSERT INTO `cachednet` (`ip`, `content`) VALUES ('$urlshort', '$contentsecure')";
                    mysqli_query($conn, $sql);
                }
                if(!empty($contents)){
                    $contentsecure = $conn->real_escape_string($content);
                    $sql = "INSERT INTO `cachednet` (`ip`, `content`) VALUES ('s$urlshort', '$contentsecure')";
                    mysqli_query($conn, $sql);
                }
            }
        }
    }
}

$conn->close();