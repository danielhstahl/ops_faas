const resolve = require('json-refs').resolveRefs
const YAML = require('yaml')
const fs = require('fs')

const root = YAML.parse(fs.readFileSync('./docs/openapi.yml').toString());
const options = {
    processContent: (content) => {
        return YAML.parse(content)
    },
    filter: ['relative'],
    location: './docs/openapi.yml'
}
module.exports = () => resolve(root, options).then((results) => {
    fs.writeFileSync(
        './docs/openapi_merged.yml',
        YAML.stringify(results.resolved)
    )
})