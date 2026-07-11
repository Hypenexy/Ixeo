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
    const defaultIcon = 'data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 64 64%22%3E%3Crect width=%2264%22 height=%2264%22 rx=%2212%22 fill=%22%2365baff%22/%3E%3Cpath d=%22M18 22h28v4H18zm0 12h28v4H18zm0 12h18v4H18z%22 fill=%22%23fff%22/%3E%3C/svg%3E';

    for (let i = 0; i < results.length; i++) {
        const result = results[i];
        const item = document.createElement('div');
        item.className = 'result-card';

        const icon = document.createElement('img');
        icon.className = 'result-icon';
        icon.src = result.image_data || defaultIcon;
        icon.alt = result.title || 'Search result icon';
        icon.loading = 'lazy';

        const content = document.createElement('div');
        content.className = 'result-content';

        const link = document.createElement('a');
        link.href = result.url;
        link.innerHTML = `<h1>${result.title}</h1>`;

        const urlLine = document.createElement('div');
        urlLine.className = 'result-url';
        urlLine.textContent = result.url;

        const description = document.createElement('p');
        description.textContent = result.description || 'This website has no description.';

        content.appendChild(link);
        content.appendChild(urlLine);
        content.appendChild(description);
        item.appendChild(icon);
        item.appendChild(content);
        resultsList.appendChild(item);
    }
});