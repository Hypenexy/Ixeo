<?php

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