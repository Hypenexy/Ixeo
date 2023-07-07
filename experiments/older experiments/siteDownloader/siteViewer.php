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
                    
                    $links = $dom->getElementsByTagName("link");
                    $styles = [];
                    foreach ($links as $file) {
                        if($file->getAttribute('rel')=="stylesheet"){
                            array_push($styles, $file);
                        }
                    }
                    $images = $dom->getElementsByTagName("img");
                    $scripts = $dom->getElementsByTagName("script");

                    function filesAtrributes($files, $attributeName){
                        foreach ($files as $file) {
                            $src = $file->getAttribute($attributeName);
                            global $folder;
                            $newsrc = $folder . '/assets/' . str_replace('/', '', $src);
                            $file->setAttribute($attributeName, $newsrc);
                        }
                    }
                    filesAtrributes($styles, "href");
                    filesAtrributes($images, "src");
                    filesAtrributes($scripts, "src");

                    // $iframes = $dom->getElementsByTagName(("iframe"));
                    // for ($i = $iframes->length; --$i >= 0;) { 
                    //     $iframes[$i]->parentNode->removeChild($iframes[$i]);
                    // }

                    echo $dom->saveXML($dom, LIBXML_NOEMPTYTAG);
                    exit;
                }
            }
        }
    }
?>