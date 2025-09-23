export function isoDateTimeSorter(a, b) {
    const dateA = new Date(a.replace(' ', 'T'));
    const dateB = new Date(b.replace(' ', 'T'));
    return dateA - dateB;
}

export function initTable(selector, columns, data) {
    const element = document.querySelector(selector);
    if (!element) return null;

    const columnsWithSorter = columns.map(col => {
        if (col.field === "date") {
            return {
                ...col,
                sorter: (a, b) => {
                    const dateA = new Date(a.replace(' ', 'T'));
                    const dateB = new Date(b.replace(' ', 'T'));
                    return dateA - dateB;
                }
            };
        }
        return col;
    });


    const table = new Tabulator(selector, {
        data: data,
        columns: columnsWithSorter,
        layout: "fitColumns",
        pagination: "local",
        paginationSize: 10,
        movableColumns: true,
        resizableRows: true,
        placeholder: "No data is available",
        autoResize: true,
        height: "auto",
    });

    return table;
}

export async function deleteFromTable(path, confirmMessage, row) {
    if (!confirm(confirmMessage)) return;

    try {
        const response = await fetch(path, { method: 'DELETE' });
        if (!response.ok) throw new Error("Error deleting from table");

        row.delete();
    } catch (err) {
        alert(err);
    }
}