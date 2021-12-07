import loadCommands from "../load_commands"
import { prefix } from "../../configs/bot_config.json"

export default {
    commands: ["help", "h", "?"],
    description: "Zayden Help Command",
    _callback: (message, arguments, text) => {
        let reply = "Zayden's Commands:\n"

        const commands = loadCommands(message.client)

        for (let command of commands) {
            let permissions = command.permissions
            let roles = command.requiredRoles
            let hasPermission = true

            // Check for permissions
            if (permissions) {
                if (message.guild == null) { continue} 

                if (typeof permissions === "string") {
                    permissions = [permissions]
                }

                for (let permission of permissions) {
                    if (!message.member.permissions.has(permission)) {
                        hasPermission = false
                        break
                    }
                }
            }

            // Check for requiredRoles
            if (roles) {
                if (message.guild == null) { continue} 
                if (typeof roles === "string") {
                    roles = [roles]
                }

                for (let requiredRole of roles) {
                    const role = message.guild.roles.cache.find(role => role.name === requiredRole)

                    if (!role || !message.member.roles.cache.has(role.id)) {
                        hasPermission = false
                        break
                    }
                }
            }

            if (!hasPermission) {
                continue
            }

            const mainCommand = typeof command.commands === "string" ? command.commands : command.commands[0]
            const args = command.expectedArgs ? ` ${command.expectedArgs}` : ""

            reply = `**${prefix}${mainCommand}${args}**\n`

        }

        message.channel.send(reply)
    },
    get callback() {
        return this._callback
    },
    set callback(value) {
        this._callback = value
    }
}