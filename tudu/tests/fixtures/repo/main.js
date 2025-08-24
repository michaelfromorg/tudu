// dummy.js - Test file for TODO parser

// === UNTRACKED TODOs (no ID) ===
// TODO: implement user authentication
// TODO: add error handling to this function
// FIXME: memory leak when processing large files
/* TODO: refactor this entire module */

// === TRACKED TODOs (with IDs) ===
// TODO(TASK-123): implement caching layer
// TODO(BUG-456): fix race condition in async handler
// TODO(FEATURE-789): add support for multiple file uploads
// FIXME(ISSUE-321): remove deprecated API calls

// === TODOs WITH ATTRIBUTES ===
// TODO(TASK-100, bidir): implement two-way sync
// TODO(BUG-200, labels=urgent,backend): fix database connection pool
// TODO(TASK-300, assignee=alice, due=2025-09-01): implement OAuth2 flow
// TODO(FEAT-400, bidir, labels=frontend,performance, assignee=bob): optimize React renders

// === EDGE CASES ===
// TODO(): empty parentheses should be untracked
// TODO(not-an-id): lowercase should be untracked
// TODO(TASK-123, labels=security,auth, close_on_delete): implement RBAC
// TODO(BUG-999, status=In Progress, prop.priority=high): custom Notion properties

// === LEGACY FORMAT (if you support it) ===
// TODO TASK-567: old style without parentheses
// FIXME BUG-890: another legacy format

// === FALSE POSITIVES (shouldn't match) ===
// This is just text with TODO in it but not a comment marker
function todoApp() {
    console.log("This TODO is inside a string, not a comment");
    let TODO = "this is a variable, not a comment";
}

// === MULTI-LINE COMMENTS ===
/* 
 * TODO(TASK-777): multi-line block comment
 * with additional description
 */

/* TODO(BUG-888, labels=critical): another block style */

// === VARIOUS COMMENT STYLES ===
// TODO(TASK-111): C++ style comment

// === COMPLEX ATTRIBUTES ===
// TODO(TASK-1000, bidir, labels=api,rest,graphql, assignee=charlie, due=2025-12-31, close_on_delete=true): full attribute test
// TODO(TASK-2000, labels=feature, status=Todo, section=parser, db=tasks): Notion-specific attributes

// === ACTUAL CODE (to make it look realistic) ===
class UserService {
    constructor() {
        // TODO(TASK-501): inject dependencies instead of direct instantiation
        this.db = new Database();
        this.cache = null; // TODO: add Redis cache here
    }

    async getUser(id) {
        // FIXME(BUG-601): add input validation
        
        // TODO(TASK-701, labels=performance): implement caching
        const user = await this.db.query(`SELECT * FROM users WHERE id = ${id}`);
        
        // TODO: hash passwords properly
        return user;
    }

    async createUser(data) {
        // TODO(TASK-801, assignee=alice, due=2025-10-15): add email verification
        
        /* TODO(BUG-901): transaction rollback not working */
        
        return this.db.insert('users', data);
    }
}
