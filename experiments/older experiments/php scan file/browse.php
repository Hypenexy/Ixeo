<?php
include('../db.php');
$conn = new mysqli($host, $user, $pass);

$result = mysqli_query($conn, "SELECT * FROM `web`");
while($row = $result->fetch_assoc()){
    $url = $row["url"];

    echo $url;
}