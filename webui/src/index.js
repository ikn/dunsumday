import m from 'mithril'

const Index = { view: () => [
    m('h1', 'Title'),
    m('p', 'Body text.'),
] }

m.route(document.body, '/index', {
    '/index': Index
})
