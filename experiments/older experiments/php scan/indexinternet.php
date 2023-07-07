<?php
include('../app/db.php');
$conn = new mysqli($host, $user, $pass, 'ixeo');
$num = 0;
ini_set('memory_limit', -1);
for ($okey = 0; $okey <= 1003; $okey++){
	$num++;
	if($num>999){
		$numstr = strval($num);
		$ipshort = $numstr[0] . "." . $numstr[1] . "." . $numstr[2] . "." . $numstr[3];
		$url = "http://" . $numstr[0] . "." . $numstr[1] . "." . $numstr[2] . "." . $numstr[3];
		$content = file_get_contents($url);

		$contentsecure = $conn->real_escape_string($content);

		$sql = "INSERT INTO `cachednet` (`ip`, `content`) VALUES ('$ipshort', '$contentsecure')";
		mysqli_query($conn, $sql);
	}
}

echo "fin";
$conn->close();


//$options = array(
//  'http'=>array(
//    'method'=>"GET",
//    'header'=>"Accept-language: en\r\n" .
//              "Cookie: foo=bar\r\n" .  // check function.stream-context-create on php.net
//              "User-Agent: Mozilla/5.0 (iPad; U; CPU OS 3_2 like Mac OS X; en-us) AppleWebKit/531.21.10 (KHTML, like Gecko) Version/4.0.4 Mobile/7B334b Safari/531.21.102011-10-16 20:23:10\r\n" // i.e. An iPad 
//  )
//);

//$context = stream_context_create($options);
//$file = file_get_contents($url, false, $context);
