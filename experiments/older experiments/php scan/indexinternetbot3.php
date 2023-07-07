<?php
include('../app/db.php');
$conn = new mysqli($host, $user, $pass, 'ixeo');
$num = 0;
error_reporting(0);
ini_set('memory_limit', -1);
ini_set('max_execution_time', '86400');
ini_set('default_socket_timeout', 2);
for ($okey = 0; $okey <= 9999; $okey++){
	$num++;
	if($num>6665&&$num<10000){
		$numstr = strval($num);
		$ipshort = $numstr[0] . "." . $numstr[1] . "." . $numstr[2] . "." . $numstr[3];
		$url = "http://" . $numstr[0] . "." . $numstr[1] . "." . $numstr[2] . "." . $numstr[3];
		$content = file_get_contents($url);
		if(!empty($content)){
			$contentsecure = $conn->real_escape_string($content);
			$sql = "INSERT INTO `cachednet` (`ip`, `content`) VALUES ('$ipshort', '$contentsecure')";
			mysqli_query($conn, $sql);
		}
	}
}
echo "fin";
$conn->close();