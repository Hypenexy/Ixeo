// Get "get" parameters from url
const urlParams = new URLSearchParams(window.location.search);
const search = urlParams.get('q');

// Elements
const searchInput = document.getElementById('q');
const resultsList = document.getElementById('resultsList');

// Change instances where the search term is used
document.title = `${search} - Ixeo`
searchInput.value = search;

window.addEventListener('DOMContentLoaded', async () => {
    if (!search || !resultsList) {
        return;
    }

    const response = await fetch(`/api/search?q=${encodeURIComponent(search)}`);
    const results = await response.json();

    // Display results
    for (let i = 0; i < results.length; i++) {
        const result = results[i];
        const item = document.createElement('div');
        const link = document.createElement('a');
        link.href = result.url;
        link.innerHTML = `<h1>${result.title}</h1>`;
        item.appendChild(link);

        if (result.image_data) {
            const img = document.createElement('img');
            img.src = result.image_data;
            img.alt = result.title;
            item.appendChild(img);
        }

        if (result.description) {
            const description = document.createElement('p');
            description.textContent = result.description;
            item.appendChild(description);
        }

        resultsList.appendChild(item);
    }
});