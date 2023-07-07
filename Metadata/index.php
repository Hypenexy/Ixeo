<?php
// Creating this to use in WriteNote's linkEngine tool
// Make sure you save in a database with caches and stuff!

// Save the content as well to make an option to view it securely.
header("Access-Control-Allow-Origin: *");

// * wont work in FF w/ Allow-Credentials
//if you dont need Allow-Credentials, * seems to work
// header('Access-Control-Allow-Origin: *');
// //if you need cookies or login etc
// header('Access-Control-Allow-Credentials: true');
// if ($this->getRequestMethod() == 'OPTIONS')
// {
//   header('Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS');
//   header('Access-Control-Max-Age: 604800');
//   //if you need special headers
//   header('Access-Control-Allow-Headers: x-requested-with');
//   exit(0);
// }


$datatosend = (object)[];
if(isset($_POST["url"])){
    $url = $_POST["url"];
    $agent= 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.93 Safari/537.36';
    if(isset($_POST["agent"])){
        $agent = $_POST["agent"];
    }
    $time = strtotime("now");

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
        $domain = explode('/', $url)[2];
        $urlimage = $domain . "/favicon.ico";
        curl_setopt($ch, CURLOPT_URL, $urlimage);
        $image = curl_exec($ch);
        if($image!=$content){
            // $urlimagefile = str_replace(["/", ":"], "", $urlimage);
            // file_put_contents("images/" . $urlimagefile, $image);
            $datatosend->image = base64_encode($image);
        }
    }
    else{
        $datatosend->status = "fail! content empty";
    }
    curl_close($ch);

    require("formatFunctions.php");
    $tags = getMetaTags($content);
    $datatosend->title = getTitle($tags, $content);
    $datatosend->alttitle = everything_in_tags($content, "h1");
    $datatosend->description = getDescription($tags);
    $datatosend->keywords = getKeywords($tags);
    $datatosend->public = getRobots($tags);
    if(isset($_POST["wholePage"])){
        $datatosend->content = $content;
    }
    
    $datatosend->text = everything_in_tags($content, "p");
    //$sql = "INSERT INTO `web` (`url`, `public`, `title`, `alttitle`, `description`, `keywords`, `text`, `code`) VALUES ('$url', '$public', '$title', '$alttitle', '$description', '$keywords', '$text', '$preserve')";

    $datatosend->status = "success";
}
else{
    $datatosend->status = "POST Parameter is not set properly or at all.";
}

echo json_encode($datatosend);