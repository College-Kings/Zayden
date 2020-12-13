const { prefix } = require("./config.json")

// commands: ping, randomise

module.exports = (client, aliases, callback) => {
    if (typeof aliases === "string") {
        aliases = [aliases]
    }

    client.on("message", message => {
        const { content } = message;

        aliases.forEach(alias => {
            const command = `${prefix}${alias}`

            if (content.startsWith(`${command} `) || content === command) {
                console.log(`Running ${command}`)
                callback(message)
            }
        })
    })
}