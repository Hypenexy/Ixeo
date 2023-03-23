<?php

?>

<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SiteViewer</title>
</head>
<body>
    <?php
        if(!isset($_GET["site"])){
            $noReleasesErr = "<div><p>There are no sites yet.</p></div>";
            if(is_dir("sitesDownloaded")){
                $styleDir = new DirectoryIterator("sitesDownloaded");
                $n = 0;
                foreach($styleDir as $fileinfo){
                    if(!$fileinfo->isDot()){
                        $n++;
                        $name = $fileinfo->getFilename();
                        $nameDeformatted = str_replace('$-', '/', $name);
                        echo "<div><a href='?site=$name'><p>$nameDeformatted</p></div>";
                    }
                }
                if($n == 0){
                    echo $noReleasesErr;
                }
            }
            else{
                echo $noReleasesErr;
            }
        }
        else{
            $folder = "sitesDownloaded/".$_GET["site"];
            $sites = new DirectoryIterator($folder);
            $n = 0;
            foreach($sites as $fileinfo){
                if(!$fileinfo->isDot()){
                    $n++;
                    $name = $fileinfo->getFilename();
                    if(substr($name, -5)==".html"){
                        $html = file_get_contents("$folder/$name");
                        
                        $dom = new DOMDocument;
                        @$dom->loadHTML($html);
                        
                        $styles = $dom->getElementsByTagName("link");
                        $images = $dom->getElementsByTagName("img");
                        $scripts = $dom->getElementsByTagName("script");

                        foreach ($styles as $style){
                            if($style->getAttribute('rel')=="stylesheet"){
                                $src = $style->getAttribute('href');
                                $newsrc = $folder . '/assets/' . str_replace('/', '', $src);
                                $style->setAttribute('href', $newsrc);
                            }
                        }
                        // foreach ($images as $image){
                        //     $src = $image->getAttribute('src');
                        //     saveAsset($url, $src, $folder);
                        // }
                        // foreach ($scripts as $script){
                        //     $src = $script->getAttribute('src');
                        //     saveAsset($url, $src, $folder);
                        // }
                        echo $dom->saveXML();
                        exit;
                    }
                }
            }
        }
    ?>
</body>
</html>