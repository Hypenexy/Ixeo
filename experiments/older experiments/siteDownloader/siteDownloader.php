<?php

function downloadFile($url){
    $ch = curl_init();
    curl_setopt($ch, CURLOPT_URL, $url);
    curl_setopt($ch, CURLOPT_VERBOSE, 1);
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, 1);
    curl_setopt($ch, CURLOPT_AUTOREFERER, false);
    curl_setopt($ch, CURLOPT_REFERER, $url);
    curl_setopt($ch, CURLOPT_HTTP_VERSION, CURL_HTTP_VERSION_1_1);
    curl_setopt($ch, CURLOPT_HEADER, 0);
    curl_setopt($ch, CURLOPT_FOLLOWLOCATION, true);
    $result = curl_exec($ch);
    curl_close($ch);
    return($result);
}

function saveFile($text, $new_filename){
    $fp = fopen($new_filename, 'w');
    fwrite($fp, $text);
    fclose($fp);
}

function saveAsset($url, $src, $folder){
    $assetURL = "$url/$src";
    echo $assetURL;
    saveFile(downloadFile($assetURL), "$folder/assets/".str_replace('/', '', $src));
    echo "<br>";
}

function getLinksInStyle($text){
    preg_match_all("/(?<=url\()[^)]+/i", $text, $matches);
    $urls = [];
	foreach ($matches[0] as $url) {
        array_push($urls, substr($url, 4, -1));
    }
    return $urls;
}

function downloadWebsite($url){
    $index = downloadFile($url);
    $urlFolder = str_replace(':', '', $url);
    $urlFolder = str_replace('/', '$-', $urlFolder);
    $folder = "sitesDownloaded/$urlFolder";
    if(!is_dir("sitesDownloaded")){
        mkdir("sitesDownloaded");
    }
    if(!is_dir($folder)){
        mkdir($folder);
        mkdir("$folder/assets");
    }
    saveFile($index, "$folder/index.html");
    $dom = new DOMDocument;
    @$dom->loadHTML($index);
    $styles = $dom->getElementsByTagName("link");
    $images = $dom->getElementsByTagName("img");
    $scripts = $dom->getElementsByTagName("script");

    foreach ($styles as $style){
        if($style->getAttribute('rel')=="stylesheet"){
            $src = $style->getAttribute('href');
            // /(?<=url\()(.*)(?=\))/gi
            $assetURL = "$url/$src";
            $urls = getLinksInStyle(downloadFile($assetURL));
            if($src!=""){
                saveAsset($url, $src, $folder);
            }
        }
    }
    foreach ($images as $image){
        $src = $image->getAttribute('src');
        if($src!=""){
            saveAsset($url, $src, $folder);
        }
    }
    foreach ($scripts as $script){
        $src = $script->getAttribute('src');
        if($src!=""){
            saveAsset($url, $src, $folder);
        }
    }
}

downloadWebsite("https://midelight.net")
?>