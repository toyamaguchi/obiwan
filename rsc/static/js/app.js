var currentPath = '/';
var directoryEntries;

(async () => {
    const getJson = async url => {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    };

    directoryEntries = await getJson('/api/v1/directory?path=' + currentPath);

    let currentPathElement = document.getElementById('current-path');
    currentPathElement.innerText = currentPath;

    let directoryEntriesElement = document.getElementById('directory-entries');
    for (let i = 0; i < directoryEntries['items'].length; i++) {
        let liElement = document.createElement('li');
        liElement.innerText = directoryEntries['items'][i]['file_name'];
        directoryEntriesElement.appendChild(liElement);
    }
})();
