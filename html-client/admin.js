const loginPanel = document.getElementById('loginPanel');
const loginForm = document.getElementById('loginForm');
const loginError = document.getElementById('loginError');
const tokenInput = document.getElementById('adminToken');
const dashboard = document.getElementById('dashboard');
const dashboardError = document.getElementById('dashboardError');
const refreshButton = document.getElementById('refreshButton');
const signOutButton = document.getElementById('signOutButton');
const updatedAt = document.getElementById('updatedAt');
const recordHead = document.getElementById('recordHead');
const recordRows = document.getElementById('recordRows');
const tableEmpty = document.getElementById('tableEmpty');
let adminToken = '';
let dashboardData;
let activeTable = 'pages';

loginForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  loginError.textContent = '';
  const candidate = tokenInput.value;
  tokenInput.value = '';
  if (!candidate) return;
  adminToken = candidate;
  await refreshDashboard(true);
});

refreshButton.addEventListener('click', () => refreshDashboard(false));
signOutButton.addEventListener('click', () => {
  adminToken = '';
  dashboardData = undefined;
  loginPanel.hidden = false;
  dashboard.hidden = true;
  refreshButton.disabled = true;
  signOutButton.hidden = true;
  updatedAt.textContent = 'Waiting for connection';
  tokenInput.focus();
});

window.setInterval(() => {
  if (adminToken) refreshDashboard(false);
}, 30000);

document.querySelectorAll('.tab').forEach((button) => {
  button.addEventListener('click', () => {
    activeTable = button.dataset.table;
    document.querySelectorAll('.tab').forEach((tab) => {
      const selected = tab === button;
      tab.classList.toggle('active', selected);
      tab.setAttribute('aria-selected', String(selected));
    });
    if (dashboardData) renderRecords(dashboardData);
  });
});

async function refreshDashboard(isLogin) {
  refreshButton.disabled = true;
  dashboardError.textContent = '';
  if (isLogin) loginError.textContent = 'Connecting...';
  try {
    const response = await fetch('/api/admin/stats', {
      headers: { Authorization: `Bearer ${adminToken}` },
      cache: 'no-store',
    });
    if (!response.ok) {
      const detail = await response.text();
      if (response.status === 401 || response.status === 503) {
        adminToken = '';
        if (isLogin) loginError.textContent = detail;
        else {
          dashboardError.textContent = detail;
          dashboardData = undefined;
          loginPanel.hidden = false;
          dashboard.hidden = true;
          signOutButton.hidden = true;
          updatedAt.textContent = 'Waiting for connection';
        }
        return;
      }
      throw new Error(detail || `Request failed (${response.status})`);
    }
    dashboardData = await response.json();
    renderDashboard(dashboardData);
    loginPanel.hidden = true;
    dashboard.hidden = false;
    signOutButton.hidden = false;
    updatedAt.textContent = `Updated ${new Date(dashboardData.generated_at).toLocaleTimeString()}`;
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Could not connect to the admin API.';
    if (isLogin) loginError.textContent = message;
    else dashboardError.textContent = message;
  } finally {
    refreshButton.disabled = !adminToken;
    if (isLogin && !adminToken) tokenInput.focus();
  }
}

function renderDashboard(data) {
  const { database, crawler, search } = data;
  setText('pagesTotal', formatNumber(database.pages_total));
  setText('pagesLastDay', `${formatNumber(database.pages_last_24h)} crawled in 24 hours`);
  setText('pagesIndexed', formatNumber(database.pages_indexed));
  setText('pagesPending', `${formatNumber(database.pages_pending)} waiting to index`);
  setText('cacheTotal', formatNumber(database.metadata_cache_total));
  setText('cacheFresh', `${formatNumber(database.metadata_cache_fresh)} fresh entries`);
  setText('searchesTotal', formatNumber(search.requests_24h));
  setText('crawlerStatus', crawler.status);
  setText('crawlerHeartbeat', crawler.heartbeat_at ? formatAge(crawler.heartbeat_at) : 'No heartbeat');
  setText('frontierSize', formatMaybeNumber(crawler.frontier_size));
  setText('crawlerTotal', formatMaybeNumber(crawler.pages_crawled_total));
  document.getElementById('crawlerDot').className = `status-dot ${crawler.status.replaceAll(' ', '-')}`;

  const total = Math.max(database.pages_total, 0);
  const indexed = Math.max(database.pages_indexed, 0);
  const percent = total === 0 ? 0 : Math.min(100, Math.round((indexed / total) * 100));
  document.getElementById('indexProgress').style.width = `${percent}%`;
  setText('indexPercent', `${percent}%`);
  setText('pipelineIndexed', formatNumber(indexed));
  setText('pipelinePending', formatNumber(database.pages_pending));
  setText('pipelineQueue', formatMaybeNumber(crawler.frontier_size));
  renderChart(search.hourly);
  renderRecords(data);
}

function renderChart(hours) {
  const chart = document.getElementById('trafficChart');
  chart.replaceChildren();
  const values = new Map(hours.map((item) => [new Date(item.hour_bucket).getTime(), item.request_count]));
  const start = Math.floor(Date.now() / 3600000) * 3600000 - 23 * 3600000;
  const buckets = Array.from({ length: 24 }, (_, index) => {
    const timestamp = start + index * 60 * 60 * 1000;
    return { timestamp, count: values.get(timestamp) || 0 };
  });
  const max = Math.max(1, ...buckets.map((bucket) => bucket.count));
  buckets.forEach((bucket, index) => {
    const bar = document.createElement('span');
    const height = bucket.count === 0 ? 2 : Math.max(7, Math.round((bucket.count / max) * 100));
    bar.className = `chart-bar${index === buckets.length - 1 ? ' latest' : ''}`;
    bar.style.height = `${height}%`;
    bar.title = `${new Date(bucket.timestamp).toLocaleString()}: ${formatNumber(bucket.count)} searches`;
    chart.appendChild(bar);
  });
}

function renderRecords(data) {
  recordHead.replaceChildren();
  recordRows.replaceChildren();
  const metadata = activeTable === 'metadata';
  const columns = metadata
    ? ['URL', 'Title', 'Fetched', 'Expires']
    : ['URL', 'Title', 'Crawled', 'Index status'];
  const headingRow = document.createElement('tr');
  columns.forEach((label) => {
    const cell = document.createElement('th');
    cell.textContent = label;
    headingRow.appendChild(cell);
  });
  recordHead.appendChild(headingRow);

  const rows = metadata ? data.recent_metadata : data.recent_pages;
  tableEmpty.hidden = rows.length > 0;
  rows.forEach((record) => {
    const row = document.createElement('tr');
    appendCell(row, record.url, true);
    appendCell(row, record.title || 'Untitled');
    if (metadata) {
      appendCell(row, formatDate(record.fetched_at), true);
      const expiry = new Date(record.expires_at);
      const statusCell = document.createElement('td');
      const status = document.createElement('span');
      status.className = `pill${expiry.getTime() <= Date.now() ? ' expired' : ''}`;
      status.textContent = expiry.getTime() <= Date.now() ? 'Expired' : 'Fresh';
      statusCell.append(status);
      row.appendChild(statusCell);
    } else {
      appendCell(row, record.crawled_at ? formatDate(record.crawled_at) : 'Unknown', true);
      const statusCell = document.createElement('td');
      const status = document.createElement('span');
      status.className = `pill${record.indexed ? '' : ' pending'}`;
      status.textContent = record.indexed ? 'Indexed' : 'Pending';
      statusCell.append(status);
      row.appendChild(statusCell);
    }
    recordRows.appendChild(row);
  });
}

function appendCell(row, value, mono = false) {
  const cell = document.createElement('td');
  if (mono) cell.className = 'mono';
  cell.textContent = value;
  row.appendChild(cell);
}

function setText(id, value) {
  document.getElementById(id).textContent = value;
}

function formatNumber(value) {
  return Number(value || 0).toLocaleString();
}

function formatMaybeNumber(value) {
  return value === null || value === undefined ? 'Unavailable' : formatNumber(value);
}

function formatDate(value) {
  return new Date(value).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' });
}

function formatAge(value) {
  const seconds = Math.max(0, Math.floor((Date.now() - new Date(value).getTime()) / 1000));
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  return `${Math.floor(seconds / 3600)}h ago`;
}