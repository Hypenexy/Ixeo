<?php
include('../db.php');
$conn = new mysqli($host, $user, $pass, 'ixeoexperiment1');

$sql = "SELECT * FROM `status`";
$result = $conn->query($sql);
$row = mysqli_fetch_assoc($result);

$time = date("F j, Y, g:i:s a", $row["started"]);
echo "Indexing: ". $row["indexing"] . "<br>";
$num = $row["indexingnum"];
$nummax = $row["indexingnummax"];
//$percentage = bcdiv($num/$nummax*100, 1) . "%";
$percentage = round($num/$nummax*100, 7) . "%";
echo "Progress: $num/$nummax <b style='color: #d0bbff;-webkit-text-fill-color:initial'>" . $percentage . "</b><br>";
echo "Started: " . $time . "<br>";//add a maximum to the status table