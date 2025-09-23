const chartInstances = {};


function getLineColors() {
    const rootStyles = getComputedStyle(document.documentElement);
    return {
        text: rootStyles.getPropertyValue('--color-text').trim(),
        grid: rootStyles.getPropertyValue('--color-rule-border').trim(),
        line: rootStyles.getPropertyValue('--color-navbar-active-bg').trim(),
        fill: rootStyles.getPropertyValue('--color-rule-active-bg').trim()
    };
}

function getPieColors() {
    const theme = document.documentElement.getAttribute("data-theme");
    const rootStyles = getComputedStyle(document.documentElement);

    return {
        text: rootStyles.getPropertyValue('--color-text').trim(),
        border: "rgba(255,255,255,0.2)",
        segments: theme === "dark"
            ? [
                "rgba(255, 99, 132, 0.8)",
                "rgba(255, 159, 64, 0.8)",
                "rgba(255, 205, 86, 0.8)",
                "rgba(100, 210, 255, 0.8)",
                "rgba(191, 90, 242, 0.8)"
            ]
            : [
                "rgba(255, 99, 132, 0.7)",
                "rgba(255, 159, 64, 0.7)",
                "rgba(255, 205, 86, 0.7)",
                "rgba(75, 192, 192, 0.7)",
                "rgba(54, 162, 235, 0.7)"
            ]
    };
}

export function createLineChart(id, labels, data, label) {
    if (chartInstances[id]) {
        chartInstances[id].destroy();
    }

    const ctx = document.getElementById(id).getContext('2d');

    let colors = getLineColors();

    chartInstances[id] = new Chart(ctx, {
        type: 'line',
        data: {
            labels,
            datasets: [{
                label: label,
                data,
                fill: true,
                backgroundColor: colors.fill,
                borderColor: colors.line,
                tension: 0.4,
                borderWidth: 2
            }]
        },
        options: {
            plugins: { legend: { labels: { color: colors.text } } },
            scales: {
                x: { ticks: { color: colors.text }, grid: { color: colors.grid } },
                y: { ticks: { color: colors.text }, grid: { color: colors.grid } }
            }
        }
    });


    const observer = new MutationObserver(() => {
        colors = getLineColors();
        chartInstances[id].data.datasets[0].backgroundColor = colors.fill;
        chartInstances[id].data.datasets[0].borderColor = colors.line;
        chartInstances[id].options.plugins.legend.labels.color = colors.text;
        chartInstances[id].options.scales.x.ticks.color = colors.text;
        chartInstances[id].options.scales.x.grid.color = colors.grid;
        chartInstances[id].options.scales.y.ticks.color = colors.text;
        chartInstances[id].options.scales.y.grid.color = colors.grid;
        chartInstances[id].update();
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });

    return chartInstances[id];
}

export function createPieChart(id, labels, data) {
    if (chartInstances[id]) {
        chartInstances[id].destroy();
    }

    const ctx = document.getElementById(id).getContext('2d');
    let colors = getPieColors();

    chartInstances[id] = new Chart(ctx, {
        type: 'pie',
        data: {
            labels,
            datasets: [{
                data,
                backgroundColor: colors.segments,
                borderColor: colors.border,
                borderWidth: 2
            }]
        },
        options: {
            plugins: {
                legend: {
                    labels: { color: colors.text }
                },
                tooltip: {
                    callbacks: {
                        label: function (context) {
                            const dataset = context.chart.data.datasets[0].data;
                            const total = dataset.reduce((a, b) => a + b, 0);
                            const value = context.parsed;
                            const percent = ((value / total) * 100).toFixed(1) + '%';
                            return `${context.label}: ${value} (${percent})`;
                        }
                    }
                }
            }
        }
    });


    const observer = new MutationObserver(() => {
        colors = getPieColors();
        chartInstances[id].data.datasets[0].backgroundColor = colors.segments;
        chartInstances[id].data.datasets[0].borderColor = colors.border;
        chartInstances[id].options.plugins.legend.labels.color = colors.text;
        chartInstances[id].update();
    });
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });

    return chartInstances[id];
}

export function createMultiLineChart(id, labels, datasets) {
    if (chartInstances[id]) {
        chartInstances[id].destroy();
    }

    const ctx = document.getElementById(id).getContext('2d');

    function getColorsForDataset(ds) {
        const rootStyles = getComputedStyle(document.documentElement);
        return {
            line: ds.color || rootStyles.getPropertyValue('--color-navbar-active-bg').trim(),
            fill: ds.fillColor || rootStyles.getPropertyValue('--color-rule-active-bg').trim()
        };
    }

    const preparedDatasets = datasets.map(ds => {
        const colors = getColorsForDataset(ds);
        return {
            label: ds.label,
            data: ds.data,
            fill: true,
            backgroundColor: colors.fill,
            borderColor: colors.line,
            tension: 0.4,
            borderWidth: 2
        };
    });

    let colors = getComputedStyle(document.documentElement);
    const textColor = colors.getPropertyValue('--color-text').trim();
    const gridColor = colors.getPropertyValue('--color-rule-border').trim();

    chartInstances[id] = new Chart(ctx, {
        type: 'line',
        data: {
            labels,
            datasets: preparedDatasets
        },
        options: {
            plugins: { legend: { labels: { color: textColor } } },
            scales: {
                x: { ticks: { color: textColor }, grid: { color: gridColor } },
                y: { ticks: { color: textColor }, grid: { color: gridColor } }
            }
        }
    });

    const observer = new MutationObserver(() => {
        const textColor = getComputedStyle(document.documentElement).getPropertyValue('--color-text').trim();
        const gridColor = getComputedStyle(document.documentElement).getPropertyValue('--color-rule-border').trim();

        chartInstances[id].options.plugins.legend.labels.color = textColor;
        chartInstances[id].options.scales.x.ticks.color = textColor;
        chartInstances[id].options.scales.x.grid.color = gridColor;
        chartInstances[id].options.scales.y.ticks.color = textColor;
        chartInstances[id].options.scales.y.grid.color = gridColor;

        chartInstances[id].data.datasets.forEach((ds, i) => {
            const colors = getColorsForDataset(datasets[i]);
            ds.backgroundColor = colors.fill;
            ds.borderColor = colors.line;
        });

        chartInstances[id].update();
    });

    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });

    return chartInstances[id];
}
