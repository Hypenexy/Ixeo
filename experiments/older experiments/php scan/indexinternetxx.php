<?php
include('../app/db.php');
$conn = new mysqli($host, $user, $pass, 'ixeo');
$num = 0;
error_reporting(0);
ini_set('memory_limit', -1);
ini_set('max_execution_time', '86400');
ini_set('default_socket_timeout', 2);
for ($okey = 0; $okey <= 100000000; $okey++){
	$num++;
	if($num>9999999&&$num<100000000){
		$numstr = strval($num);
		$ipshort = $numstr[0] . $numstr[1] . "." . $numstr[2] . $numstr[3] . "." . $numstr[4] . $numstr[5] . "." . $numstr[6] . $numstr[7];
		$url = "http://" . $numstr[0] . $numstr[1] . "." . $numstr[2] . $numstr[3] . "." . $numstr[4] . $numstr[5] . "." . $numstr[6] . $numstr[7];
		//$content = file_get_contents($url);
        $ch = curl_init();
        curl_setopt($ch, CURLOPT_URL, $url);
        curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
        $content = curl_exec($ch);
        curl_close($ch);
		if(!empty($content)){
			$contentsecure = $conn->real_escape_string($content);
			$sql = "INSERT INTO `cachednet` (`ip`, `content`) VALUES ('$ipshort', '$contentsecure')";
			mysqli_query($conn, $sql);
		}
	}
}
echo "fin";
$conn->close();