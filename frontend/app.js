// API base URL
const API_BASE = 'http://127.0.0.1:8080';

// API call wrapper
async function apiCall(endpoint, method = 'GET', body = null, requiresAuth = false) {
    const options = {
        method,
        headers: {
            'Content-Type': 'application/json',
        },
    };

    // Add authorization token if required
    if (requiresAuth) {
        const token = localStorage.getItem('token');
        if (token) {
            options.headers['Authorization'] = `Bearer ${token}`;
        }
    }

    // Add body for POST/PUT requests
    if (body && (method === 'POST' || method === 'PUT')) {
        options.body = JSON.stringify(body);
    }

    try {
        const response = await fetch(`${API_BASE}${endpoint}`, options);
        
        // Handle different response types
        const contentType = response.headers.get('content-type');
        let data;
        
        if (contentType && contentType.includes('application/json')) {
            data = await response.json();
        } else {
            data = await response.text();
        }

        if (!response.ok) {
            // If unauthorized, redirect to login
            if (response.status === 401) {
                localStorage.removeItem('token');
                localStorage.removeItem('user');
                window.location.href = '/';
                throw new Error('Session expired. Please login again.');
            }
            
            throw new Error(data.error || data.message || `HTTP ${response.status}: ${response.statusText}`);
        }

        return data;
    } catch (error) {
        console.error('API call failed:', error);
        throw error;
    }
}

// Check if user is authenticated and has correct role
function checkAuth(requiredRole = null) {
    const token = localStorage.getItem('token');
    const userStr = localStorage.getItem('user');
    
    if (!token || !userStr) {
        window.location.href = '/';
        return false;
    }
    
    try {
        const user = JSON.parse(userStr);
        
        if (requiredRole && user.role !== requiredRole) {
            alert('Access denied. Insufficient permissions.');
            handleLogout();
            return false;
        }
        
        return true;
    } catch (e) {
        localStorage.removeItem('token');
        localStorage.removeItem('user');
        window.location.href = '/';
        return false;
    }
}

// Logout function
async function handleLogout() {
    const token = localStorage.getItem('token');
    
    if (token) {
        try {
            await apiCall('/api/auth/logout', 'POST', {}, true);
        } catch (error) {
            console.error('Logout error:', error);
        }
    }
    
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    window.location.href = '/';
}

// Utility function to format dates
function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
    });
}

// Utility function to format date and time
function formatDateTime(dateString) {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

// Calculate days between two dates
function daysBetween(date1, date2) {
    const oneDay = 24 * 60 * 60 * 1000;
    const firstDate = new Date(date1);
    const secondDate = new Date(date2);
    
    return Math.round(Math.abs((firstDate - secondDate) / oneDay));
}

// Check if date is overdue
function isOverdue(dueDate) {
    return new Date(dueDate) < new Date();
}
