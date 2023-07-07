<?php
ini_set('memory_limit', -1);
ini_set('max_execution_time', '0');
set_time_limit(0);
ini_set('default_socket_timeout', 2);

function everything_in_tags($string, $tagname)
{
    $pattern = "#<\s*?$tagname\b[^>]*>(.*?)</$tagname\b[^>]*>#s";
    preg_match($pattern, $string, $matches);
    if(!isset($matches[1])){
        return "";
    }
    return $matches[1];
}

function getMetaTags($str)
{
  $pattern = '
  ~<\s*meta\s

  # using lookahead to capture type to $1
    (?=[^>]*?
    \b(?:name|property|http-equiv)\s*=\s*
    (?|"\s*([^"]*?)\s*"|\'\s*([^\']*?)\s*\'|
    ([^"\'>]*?)(?=\s*/?\s*>|\s\w+\s*=))
  )

  # capture content to $2
  [^>]*?\bcontent\s*=\s*
    (?|"\s*([^"]*?)\s*"|\'\s*([^\']*?)\s*\'|
    ([^"\'>]*?)(?=\s*/?\s*>|\s\w+\s*=))
  [^>]*>

  ~ix';
 
  if(preg_match_all($pattern, $str, $out)){
      
    foreach ($out[1] as &$value) {
        $value = strtolower($value);
    }

    return array_combine($out[1], $out[2]);
  }
  return array();
}
function getTitle($tags, $content){
    if(isset($tags['title'])){
        return $tags['title'];
    }
    else if(isset($tags['og:title'])){
        return $tags['og:title'];
    }
    else if(isset($tags['twitter:title'])){
        return $tags['twitter:title'];
    }
    else {
        return everything_in_tags($content, "title");
    }
    return "";
}
function getRobots($tags){
    if(isset($tags['robots'])){
        if(strpos($tags['robots'], 'noindex')!== false)
        return 0;
    }
    return 1;
}

function getKeywords($tags){
    if(isset($tags['keywords'])){
        return $tags['keywords'];
    }
    return "";
}

function getDescription($tags){
    if(isset($tags['description'])){
        return $tags['description'];
    }
    else if(isset($tags['og:description'])){
        return $tags['og:description'];
    }
    else if(isset($tags['dc.description'])){
        return $tags['dc.description'];
    }
    else if(isset($tags['twitter:description'])){
        return $tags['twitter:description'];
    }
    else if(isset($tags['twitter:text:description'])){
        return $tags['twitter:text:description'];
    }
    // else{
    //     echo "<textarea style='width:100%;height:200px'>";
    //     print_r($tags);
    //     echo "</textarea>";
    // }
    return "";
}

include('../db.php');
$conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');

$sql = "DROP TABLE `web`";
if ($conn->query($sql) === TRUE) echo 'Data deleted<br>';
else{echo $conn->error."<br>";}
$sql = "CREATE TABLE `ixeoExperiment2`.`web` ( `url` VARCHAR(400) NOT NULL , `public` INT(2) NOT NULL , `title` LONGTEXT NOT NULL , `alttitle` LONGTEXT NOT NULL , `description` LONGTEXT NOT NULL , `keywords` LONGTEXT NOT NULL , `text` LONGTEXT NOT NULL , `code` LONGTEXT NOT NULL ) ENGINE = InnoDB;";
if ($conn->query($sql) === TRUE) echo 'Table successfully created<br>';
else{echo $conn->error."<br>";}

$result = mysqli_query($conn, "SELECT * FROM `cachednet`");
while($row = $result->fetch_assoc()){
    $conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');
    $url = $row["url"];
    $preserve =  $conn->real_escape_string($row["content"]);
    $content = nl2br(stripslashes($row["content"]));
    $tags = getMetaTags($content);
    $title = $conn->real_escape_string(getTitle($tags, $content));
    $alttitle = $conn->real_escape_string(everything_in_tags($content, "h1"));
    $description = $conn->real_escape_string(getDescription($tags));
    $keywords = $conn->real_escape_string(getKeywords($tags));
    $public = getRobots($tags);
    
    $text = $conn->real_escape_string(everything_in_tags($content, "p"));
    $sql = "INSERT INTO `web` (`url`, `public`, `title`, `alttitle`, `description`, `keywords`, `text`, `code`) VALUES ('$url', '$public', '$title', '$alttitle', '$description', '$keywords', '$text', '$preserve')";
    mysqli_query($conn, $sql);
    $conn->close();
}

$conn = new mysqli($host, $user, $pass, 'ixeoexperiment2');

if ($conn->query($sql) === TRUE) echo "Maybe it ended succesfully<br>";
else{echo $conn->error."<br>";}

echo "bye";