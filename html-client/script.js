// Get "get" parameters from url
const urlParams = new URLSearchParams(window.location.search);
const search = urlParams.get('q');

// Elements
const searchInput = document.getElementById('q');
const resultsList = document.getElementById('resultsList');

// Change instances where the search term is used
document.title = `${search} - Ixeo`
searchInput.value = search;

;(async () => {
    if (!search) {
        return;
    }

    const response = await fetch(`/api/search?q=${encodeURIComponent(search)}`);
    const results = await response.json();

    // Display results
    for (let i = 0; i < results.length; i++) {
        const result = results[i];
        const listItem = document.createElement('li');
        const link = document.createElement('a');
        link.href = result.url;
        link.textContent = result.title;
        listItem.appendChild(link);
        resultsList.appendChild(listItem);
    }
})();