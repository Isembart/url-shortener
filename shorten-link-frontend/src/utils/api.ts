import axios from "axios";
export const API = axios.create();

export const API_URL = import.meta.env.VITE_API_URL || window.location.origin;
API.defaults.baseURL=API_URL;
