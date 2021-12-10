import { ReactionRole } from "./reactionRole"

export class Server {
    id: string;
    reactionRoles: ReactionRole[]
    disabledCommands: string[];
    moderationRole: string;
    channels: Record<string, string>;
    gameVersions: Record<string, string>
    serverRules: Record<string, string>
    serverGuidelines: Array<Array<string>>
    idNumber: number;
    hidden: Record<string, Record<string, string>>


    constructor(id: string) {
        this.id = id;
        this.reactionRoles = []
        this.disabledCommands = []
        this.moderationRole = ""
        this.channels = {};
        this.idNumber = 0;
        this.gameVersions = {}
        this.serverRules = {}
        this.serverGuidelines = [];
        this.hidden = {}

        servers[this.id] = this
    }
}

export let servers: Record<string, Server> = {};